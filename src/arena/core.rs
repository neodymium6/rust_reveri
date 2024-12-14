use std::{io::{BufRead, BufReader, Write}, process::{Child, ChildStdin, ChildStdout, Command}};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

use crate::board::core::{Board, BoardError, Turn};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);


#[derive(Debug)]
pub enum PlayerError {
    NotStarted,
    IoError,
    ParseError,
    TimeoutError,
    BoardError,
}

struct Player {
    command: Vec<String>,
    turn: Turn,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

impl Player {
    fn new(command: Vec<String>, turn: Turn) -> Self {
        Player {
            command,
            turn,
            process: None,
            stdin: None,
            stdout: None,
        }
    }

    fn start(&mut self) -> Result<(), std::io::Error> {
        let mut command = Command::new(&self.command[0]);
        for arg in self.command.iter().skip(1) {
            command.arg(arg);
        }

        match self.turn {
            Turn::Black => command.arg("BLACK"),
            Turn::White => command.arg("WHITE"),
        };

        let mut process = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        self.process = Some(process);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);

        Ok(())
    }

    fn stop(&mut self) -> Result<(), std::io::Error> {
        if let Some(process) = &mut self.process {
            process.kill()?;
            process.wait()?;
        }
        self.process = None;
        self.stdin = None;
        self.stdout = None;
        Ok(())
    }

    fn get_move_with_timeout(&mut self, board: &Board, timeout: Duration) -> Result<usize, PlayerError> {
        let (tx, rx) = mpsc::channel();
        let stdin = self.stdin.as_mut().ok_or(PlayerError::NotStarted)?;
        let board_line = board.get_board_line()
            .map_err(|e| match e {
                _ => PlayerError::BoardError,
            })?;
        writeln!(stdin, "{}", board_line)
            .map_err(|_| PlayerError::IoError)?;
        stdin.flush()
            .map_err(|_| PlayerError::IoError)?;

        let stdout = self.stdout.take().ok_or(PlayerError::NotStarted)?;
        let mut reader = stdout;
        
        thread::spawn(move || {
            let mut response = String::new();
            let result = reader.read_line(&mut response)
                .map_err(|_| PlayerError::IoError)
                .and_then(|_| response.trim().parse::<usize>()
                    .map_err(|_| PlayerError::ParseError));
            let _ = tx.send((result, reader));
        });

        match rx.recv_timeout(timeout) {
            Ok((result, reader)) => {
                self.stdout = Some(reader);
                result
            }
            Err(_) => Err(PlayerError::TimeoutError)
        }
    }
}

#[derive(Debug, Clone)]
enum GameResult {
    BlackWin(usize, usize),
    WhiteWin(usize, usize),
    Draw(usize, usize),
}

#[derive(Debug, Clone)]
enum GameStatus {
    FINISHED(GameResult),
    Playing,
}

#[derive(Debug)]
pub enum GameError {
    BlackInvalidMove,
    WhiteInvalidMove,
    BlackTimeout,
    WhiteTimeout,
    BlackCrash,
    WhiteCrash,
    UnexpectedError,
}

struct Game<'a> {
    board: Board,
    black_player: &'a mut Player,
    white_player: &'a mut Player,
    moves: Vec<Option<usize>>,
    board_log: Vec<(u64, u64, Turn)>,
    status: GameStatus,
}

impl<'a> Game<'a> {
    fn new(black_player: &'a mut Player, white_player: &'a mut Player) -> Self {
        Game {
            board: Board::new(),
            black_player,
            white_player,
            moves: Vec::new(),
            board_log: Vec::new(),
            status: GameStatus::Playing,
        }
    }

    fn get_move(&mut self) -> Result<usize, GameError> {
        let player = match self.board.get_board().2 {
            Turn::Black => &mut self.black_player,
            Turn::White => &mut self.white_player,
        };
        player.get_move_with_timeout(&self.board, DEFAULT_TIMEOUT)
            .map_err(|e| match e {
                PlayerError::NotStarted => match self.board.get_board().2 {
                    Turn::Black => GameError::UnexpectedError,
                    Turn::White => GameError::UnexpectedError,
                },
                PlayerError::IoError => match self.board.get_board().2 {
                    Turn::Black => GameError::BlackCrash,
                    Turn::White => GameError::WhiteCrash,
                },
                PlayerError::ParseError => match self.board.get_board().2 {
                    Turn::Black => GameError::BlackInvalidMove,
                    Turn::White => GameError::WhiteInvalidMove,
                },
                PlayerError::TimeoutError => match self.board.get_board().2 {
                    Turn::Black => GameError::BlackTimeout,
                    Turn::White => GameError::WhiteTimeout,
                },
                PlayerError::BoardError => GameError::UnexpectedError,
            })
    }

    fn play(&mut self) -> Result<(), GameError> {
        while !self.board.is_game_over() {
            if self.board.is_pass() {
                self.board.do_pass().unwrap();
                self.moves.push(None);
                continue;
            }
            let mv = self.get_move()
                .map_err(|e| return e)?;
            self.board.do_move(mv)
                .map_err(|e| match e {
                    BoardError::InvalidMove => match self.board.get_board().2 {
                        Turn::Black => return GameError::BlackInvalidMove,
                        Turn::White => return GameError::WhiteInvalidMove,
                    },
                    BoardError::InvalidPosition => match self.board.get_board().2 {
                        Turn::Black => return GameError::BlackInvalidMove,
                        Turn::White => return GameError::WhiteInvalidMove,
                    },
                    _ => return GameError::UnexpectedError,
                })?;
                self.moves.push(Some(mv));
                self.board_log.push(self.board.get_board());
        }

        let winner = self.board.get_winner().unwrap();
        let black_pieces = self.board.black_piece_num() as usize;
        let white_pieces = self.board.white_piece_num() as usize;
        self.status = match winner {
            Some(Turn::Black) => GameStatus::FINISHED(GameResult::BlackWin(black_pieces, white_pieces)),
            Some(Turn::White) => GameStatus::FINISHED(GameResult::WhiteWin(black_pieces, white_pieces)),
            None => GameStatus::FINISHED(GameResult::Draw(black_pieces, white_pieces)),
        };
        Ok(())
    }

    fn get_result(&self) -> GameResult {
        match &self.status {
            GameStatus::FINISHED(result) => result.clone(),
            _ => panic!("Game is not finished yet"),
        }
    }
}

pub enum ArenaError {
    EngineStartError,
    GameNumberInvalid,
    GameError(GameError),
}

pub enum PlayerOrder {
    P1equalsBlack,
    P2equalsBlack,
}
pub struct Arena {
    games: Vec<(PlayerOrder, GameResult)>,
    command1: Vec<String>,
    command2: Vec<String>,
}

impl Arena {
    pub fn new(command1: Vec<String>, command2: Vec<String>) -> Self {
        Arena {
            games: Vec::new(),
            command1,
            command2,
        }
    }

    pub fn play_n(&mut self, n: usize) -> Result<(), ArenaError> {
        if n % 2 != 0 {
            return Err(ArenaError::GameNumberInvalid);
        }
        let mut p1_black_player = Player::new(self.command1.clone(), Turn::Black);
        let mut p1_white_player = Player::new(self.command2.clone(), Turn::White);
        let mut p2_black_player = Player::new(self.command2.clone(), Turn::Black);
        let mut p2_white_player = Player::new(self.command1.clone(), Turn::White);
        p1_black_player.start()
            .map_err(|_| ArenaError::EngineStartError)?;
        p1_white_player.start()
            .map_err(|_| ArenaError::EngineStartError)?;
        p2_black_player.start()
            .map_err(|_| ArenaError::EngineStartError)?;
        p2_white_player.start()
            .map_err(|_| ArenaError::EngineStartError)?;
    
        for _ in 0..(n / 2) {
            {
                let mut game = Game::new(&mut p1_black_player, &mut p2_white_player);
                game.play()
                    .map_err(|e| ArenaError::GameError(e))?;
                
                self.games.push((PlayerOrder::P1equalsBlack, game.get_result()));
            }

            {
                let mut game = Game::new(&mut p2_black_player, &mut p1_white_player);
                game.play()
                    .map_err(|e| ArenaError::GameError(e))?;
            
                self.games.push((PlayerOrder::P2equalsBlack, game.get_result()));
            }
        }
        Ok(())
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        let mut p1_win = 0;
        let mut p2_win = 0;
        let mut draw = 0;
        for (order, game_result) in self.games.iter() {
            match game_result {
                GameResult::BlackWin(_, _) => {
                    match order {
                        PlayerOrder::P1equalsBlack => p1_win += 1,
                        PlayerOrder::P2equalsBlack => p2_win += 1,
                    }
                },
                GameResult::WhiteWin(_, _) => {
                    match order {
                        PlayerOrder::P1equalsBlack => p2_win += 1,
                        PlayerOrder::P2equalsBlack => p1_win += 1,
                    }
                },
                GameResult::Draw(_, _) => draw += 1,
            }
        }
        (p1_win, p2_win, draw)
    }

    pub fn get_pieces(&self) -> (usize, usize) {
        let mut p1_pieces = 0;
        let mut p2_pieces = 0;
        for (order, game_result) in self.games.iter() {
            match game_result {
                GameResult::BlackWin(black_pieces, white_pieces) => {
                    match order {
                        PlayerOrder::P1equalsBlack => {
                            p1_pieces += black_pieces;
                            p2_pieces += white_pieces;
                        }
                        PlayerOrder::P2equalsBlack => {
                            p1_pieces += white_pieces;
                            p2_pieces += black_pieces;
                        }
                    }
                },
                GameResult::WhiteWin(black_pieces, white_pieces) => {
                    match order {
                        PlayerOrder::P1equalsBlack => {
                            p2_pieces += white_pieces;
                            p1_pieces += black_pieces;
                        }
                        PlayerOrder::P2equalsBlack => {
                            p2_pieces += black_pieces;
                            p1_pieces += white_pieces;
                        }
                    }
                },
                GameResult::Draw(black_pieces, white_pieces) => {
                    p1_pieces += black_pieces;
                    p2_pieces += white_pieces;
                },
            }
        }
        (p1_pieces, p2_pieces)
    }
}
