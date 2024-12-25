use crate::arena::core::{Arena, Player};
use crate::arena::error::ArenaError;
use crate::board::core::Turn;
use std::process::Stdio;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command},
};

#[derive(Debug)]
pub struct LocalArena {
    command1: Vec<String>,
    command2: Vec<String>,
    stats: (usize, usize, usize),
    pieces: (usize, usize),
}

type ProcessPlayer = Player<ChildStdin, BufReader<ChildStdout>>;
type ProcessTuple = (Child, ChildStdin, BufReader<ChildStdout>);
type ProcessResult = Result<(ProcessTuple, ProcessTuple), ArenaError>;
type ProcessPair = (Child, Child);
type PlayerPair = (ProcessPlayer, ProcessPlayer);
impl LocalArena {
    pub fn new(command1: Vec<String>, command2: Vec<String>) -> Self {
        LocalArena {
            command1,
            command2,
            stats: (0, 0, 0),
            pieces: (0, 0),
        }
    }

    fn start_process(
        command: &[String],
        turn: Turn,
    ) -> Result<(Child, ChildStdin, BufReader<ChildStdout>), std::io::Error> {
        let mut cmd = Command::new(&command[0]);
        for arg in command.iter().skip(1) {
            cmd.arg(arg);
        }

        match turn {
            Turn::Black => cmd.arg("BLACK"),
            Turn::White => cmd.arg("WHITE"),
        };

        let mut process = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

        let mut stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        // ping-pong test
        writeln!(stdin, "ping")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Write error"))?;
        stdin
            .flush()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Flush error"))?;

        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Read error"))?;

        if response.trim() != "pong" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid response",
            ));
        }

        Ok((process, stdin, reader))
    }

    fn init_processes(&self, p1_turn: Turn) -> ProcessResult {
        let (process1, stdin1, stdout1) = Self::start_process(&self.command1, p1_turn)
            .map_err(|_| ArenaError::EngineStartError)?;

        let p2_turn = p1_turn.opposite();
        let (process2, stdin2, stdout2) = Self::start_process(&self.command2, p2_turn)
            .map_err(|_| ArenaError::EngineStartError)?;

        Ok(((process1, stdin1, stdout1), (process2, stdin2, stdout2)))
    }

    fn get_players(&mut self) -> Result<(Vec<ProcessPair>, Vec<PlayerPair>), ArenaError> {
        // P1equalsBlack
        let ((process1, stdin1, stdout1), (process2, stdin2, stdout2)) =
            self.init_processes(Turn::Black)?;
        let player_1b = Player::new(stdin1, stdout1);
        let player_2w = Player::new(stdin2, stdout2);

        // P2equalsBlack
        let ((process3, stdin3, stdout3), (process4, stdin4, stdout4)) =
            self.init_processes(Turn::White)?;
        let player_2b = Player::new(stdin4, stdout4);
        let player_1w = Player::new(stdin3, stdout3);

        Ok((
            vec![(process1, process2), (process3, process4)],
            vec![(player_1b, player_2w), (player_2b, player_1w)],
        ))
    }

    pub fn play_n(&mut self, n: usize) -> Result<(), ArenaError> {
        let (mut processes, players) = self.get_players()?;

        let mut arena = Arena::new(players);
        arena.play_n(n)?;
        let (p1_win, p2_win, draw) = arena.get_stats();
        self.stats.0 += p1_win;
        self.stats.1 += p2_win;
        self.stats.2 += draw;
        let (p1_pieces, p2_pieces) = arena.get_pieces();
        self.pieces.0 += p1_pieces;
        self.pieces.1 += p2_pieces;

        // drop all processes
        for (p1, p2) in processes.iter_mut() {
            p1.kill().map_err(|_| ArenaError::EngineEndError)?;
            p1.wait().map_err(|_| ArenaError::EngineEndError)?;
            p2.kill().map_err(|_| ArenaError::EngineEndError)?;
            p2.wait().map_err(|_| ArenaError::EngineEndError)?;
        }

        Ok(())
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        self.stats
    }

    pub fn get_pieces(&self) -> (usize, usize) {
        self.pieces
    }
}
