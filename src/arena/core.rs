use std::io::{BufRead, Write};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};
use tqdm::pbar;
use crate::board::core::{Board, BoardError, Turn};
use crate::arena::error::{ArenaError, GameError, PlayerError};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);


pub struct Player<W, R>
where
    W: Write + Send + 'static,
    R: BufRead + Send + 'static,
{
    stdin: W,
    stdout: Arc<Mutex<R>>,
}

impl<W, R> Player<W, R>
where
    R: BufRead + Send + 'static,
    W: Write + Send + 'static,
{
    pub fn new(stdin: W, stdout: R) -> Self {
        Player {
            stdin,
            stdout: Arc::new(Mutex::new(stdout)),
        }
    }

    pub fn get_move_with_timeout(
        &mut self,
        board: &Board,
        timeout: Duration,
    ) -> Result<usize, PlayerError> {
        let (tx, rx) = mpsc::channel();
        
        let mut board_line = board
            .get_board_line()
            .map_err(|_| PlayerError::BoardError)?;
        board_line.push('\n');
            
        self.stdin.write(board_line.as_bytes()).unwrap();
        self.stdin
            .flush()
            .map_err(|_| PlayerError::IoError)?;

        let stdout = Arc::clone(&self.stdout);
        
        let handle = thread::spawn(move || {
            let mut stdout = stdout.lock().unwrap();
            let mut response = String::new();
            let result = stdout
                .read_line(&mut response)
                .map_err(|_| PlayerError::IoError)
                .and_then(|_| {
                    response.trim().parse::<usize>()
                        .map_err(|_| PlayerError::ParseError)
                });
            tx.send(result).unwrap();
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => {
                handle.join().unwrap();
                result
            }
            Err(_) => {
                handle.thread().unpark();
                Err(PlayerError::TimeoutError)
            }
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

struct Game<'a, W, R>
where
    W: Write + Send + 'static,
    R: BufRead + Send + 'static,
{
    board: Board,
    black_player: &'a mut Player<W, R>,
    white_player: &'a mut Player<W, R>,
    moves: Vec<Option<usize>>,
    board_log: Vec<(u64, u64, Turn)>,
    status: GameStatus,
}

impl<'a, W, R> Game<'a, W, R>
where
    W: Write + Send + 'static,
    R: BufRead + Send + 'static,
{
    fn new(
        black_player: &'a mut Player<W, R>,
        white_player: &'a mut Player<W, R>,
    ) -> Self {
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
        let turn = self.board.get_board().2;
        let player = match turn {
            Turn::Black => &mut self.black_player,
            Turn::White => &mut self.white_player,
        };
        player.get_move_with_timeout(&self.board, DEFAULT_TIMEOUT)
            .map_err(|e| match e {
                PlayerError::IoError => match turn {
                    Turn::Black => GameError::BlackCrash,
                    Turn::White => GameError::WhiteCrash,
                },
                PlayerError::ParseError => match turn {
                    Turn::Black => GameError::BlackInvalidMove,
                    Turn::White => GameError::WhiteInvalidMove,
                },
                PlayerError::TimeoutError => match turn {
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

    fn get_result(&self) -> Result<GameResult, GameError> {
        match &self.status {
            GameStatus::FINISHED(result) => Ok(result.clone()),
            GameStatus::Playing => Err(GameError::GameNotOverYet),
        }
    }
}


pub enum PlayerOrder {
    P1equalsBlack,
    P2equalsBlack,
}
pub struct Arena<W, R>
where
    W: Write + Send + 'static,
    R: BufRead + Send + 'static,
{
    games: Vec<(PlayerOrder, GameResult)>,
    players: Vec<Arc<Mutex<(Player<W, R>, Player<W, R>)>>>,
}

impl <W, R> Arena<W, R>
where
    W: Write + Send + 'static,
    R: BufRead + Send + 'static,
{
    pub fn new(players: Vec<(Player<W, R>, Player<W, R>)>) -> Self {
        Arena {
            games: Vec::new(),
            players: players.into_iter().map(|(p1, p2)| Arc::new(Mutex::new((p1, p2)))).collect(),
        }
    }

    pub fn play_n(&mut self, n: usize) -> Result<(), ArenaError> {
        if n % 2 != 0 {
            return Err(ArenaError::GameNumberInvalid);
        }

        let half_n = n / 2;
        let players0 = Arc::clone(&self.players[0]);
        let players1 = Arc::clone(&self.players[1]);

        let pb = Arc::new(Mutex::new(pbar(Some(n))));
        let pb1 = Arc::clone(&pb);
        let pb2 = Arc::clone(&pb);

        let mut handles = vec![];
        // p1equalsBlack
        {
            handles.push(thread::spawn(move || {
                let mut results = Vec::with_capacity(half_n);
                for _ in 0..half_n {
                    let mut player0 = players0.lock().unwrap();
                    let (p1, p2) = &mut *player0;
                    let mut game = Game::new(p1, p2);
                    match game.play() {
                        Ok(_) => {
                            let result = game.get_result();
                            results.push((PlayerOrder::P1equalsBlack, result));
                        }
                        Err(e) => return Err(ArenaError::GameError(e)),
                    }
                    pb1.lock().unwrap().update(1).unwrap();
                }
                Ok(results)
            }));
        }

        // p2equalsBlack
        {
            handles.push(thread::spawn(move || {
                let mut results = Vec::with_capacity(half_n);
                for _ in 0..half_n {
                    let mut player1 = players1.lock().unwrap();
                    let (p1, p2) = &mut *player1;
                    let mut game = Game::new(p1, p2);
                    match game.play() {
                        Ok(_) => {
                            let result = game.get_result();
                            results.push((PlayerOrder::P2equalsBlack, result));
                        }
                        Err(e) => return Err(ArenaError::GameError(e)),
                    }
                    pb2.lock().unwrap().update(1).unwrap();
                }
                Ok(results)
            }));
        }

        let mut all_results = Vec::with_capacity(n);
        for handle in handles {
            match handle.join() {
                Ok(Ok(mut results)) => {
                    for result in results.drain(..) {
                        match result {
                            (order, Ok(result)) => all_results.push((order, result)),
                            (_, Err(e)) => {
                                return Err(ArenaError::GameError(e));
                            }
                            
                        }
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(ArenaError::ThreadJoinError),
            }
        }
        self.games.extend(all_results);
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
