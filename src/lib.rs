use std::mem::swap;

use pyo3::{exceptions::PyValueError, prelude::*};

const BOARD_SIZE: usize = 8;
const LINE_CHAR_BLACK: char = 'X';
const LINE_CHAR_WHITE: char = 'O';
const LINE_CHAR_EMPTY: char = '-';

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
enum Turn {
    Black,
    White, 
}

impl Turn {
    fn opposite(&self) -> Turn {
        match self {
            Turn::Black => Turn::White,
            Turn::White => Turn::Black,
        }
    }
}

#[pymethods]
impl Turn {
    fn __str__(&self) -> &'static str {
        match self {
            Turn::Black => "Black",
            Turn::White => "White",
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
enum Color {
    Empty,
    Black,
    White,
}

#[pymethods]
impl Color {
    fn __str__(&self) -> &'static str {
        match self {
            Color::Empty => "Empty",
            Color::Black => "Black",
            Color::White => "White",
        }
    }
}

#[pyclass]
struct Board {
    player_board: u64,
    opponent_board: u64,
    turn: Turn,
}

#[pymethods]
impl Board {
    #[new]
    fn new() -> Self {
        Board {
            player_board: 0x00_00_00_08_10_00_00_00,
            opponent_board: 0x00_00_00_10_08_00_00_00,
            turn: Turn::Black,
        }
    }

    #[staticmethod]
    fn pos2bit(pos: usize) -> u64 {
        // 0 -> left-top, 63 -> right-bottom
        1 << (BOARD_SIZE * BOARD_SIZE - 1 - pos)
    }

    fn get_board(&self) -> (u64, u64, Turn) {
        (self.player_board, self.opponent_board, self.turn)
    }

    fn set_board(&mut self, player_board: u64, opponent_board: u64, turn: Turn) {
        self.player_board = player_board;
        self.opponent_board = opponent_board;
        self.turn = turn;
    }

    fn set_board_str(&mut self, line: &str, turn: Turn) -> PyResult<()> {
        let mut player_board: u64 = 0;
        let mut opponent_board: u64 = 0;
        for (i, c) in line.chars().enumerate() {
            let pos = Board::pos2bit(i);
            match c {
                LINE_CHAR_BLACK => {
                    player_board |= pos;
                }
                LINE_CHAR_WHITE => {
                    opponent_board |= pos;
                }
                LINE_CHAR_EMPTY => {}
                _ => {
                    return Err(PyValueError::new_err("Invalid character"));
                }
            }
        }
        if turn == Turn::Black {
            self.player_board = player_board;
            self.opponent_board = opponent_board;
        } else {
            self.player_board = opponent_board;
            self.opponent_board = player_board;
        }
        self.turn = turn;
        Ok(())
    }

    fn get_board_vec(&self) -> PyResult<Vec<i32>> {
        let mut board_vec = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                match (self.player_board & pos == 0, self.opponent_board & pos == 0) {
                    (true, true) => board_vec.push(Color::Empty as i32),    // Empty
                    (false, true) => board_vec.push(Color::Black as i32),   // Player
                    (true, false) => board_vec.push(Color::White as i32),   // Opponent
                    _ => {
                        return Err(PyValueError::new_err("Invalid board state"));
                    }
                }
            }
        }
        Ok(board_vec)
    }

    fn get_board_matrix(&self) -> PyResult<Vec<Vec<Vec<i32>>>> {
        let mut board_matrix = vec![vec![vec![0; BOARD_SIZE]; BOARD_SIZE]; 3];
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                match (self.player_board & pos == 0, self.opponent_board & pos == 0) {
                    (true, true) => board_matrix[2][i][j] = 1,  // Empty
                    (false, true) => board_matrix[0][i][j] = 1, // Player
                    (true, false) => board_matrix[1][i][j] = 1, // Opponent
                    _ => {
                        return Err(PyValueError::new_err("Invalid board state"));
                    }
                }
            }
        }
        Ok(board_matrix)
    }

    fn player_piece_num(&self) -> i32 {
        self.player_board.count_ones() as i32
    }

    fn opponent_piece_num(&self) -> i32 {
        self.opponent_board.count_ones() as i32
    }

    fn black_piece_num(&self) -> i32 {
        if self.turn == Turn::Black {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    fn white_piece_num(&self) -> i32 {
        if self.turn == Turn::White {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    fn piece_sum(&self) -> i32 {
        self.player_piece_num() + self.opponent_piece_num()
    }

    fn diff_piece_num(&self) -> i32 {
        (self.player_piece_num() - self.opponent_piece_num()).abs()
    }

    fn get_legal_moves(&self) -> u64 {
        let horizontal_watch = 0x7E_7E_7E_7E_7E_7E_7E_7E & self.opponent_board;
        let vertical_watch = 0x00_FF_FF_FF_FF_FF_FF_00 & self.opponent_board;
        let all_watch = 0x_00_7E_7E_7E_7E_7E_7E_7E_00 & self.opponent_board;
        let blank = !(self.player_board | self.opponent_board);
        let mut legal = 0x00_00_00_00_00_00_00_00;

        // max of number of stones to reverse in each direction is 6
        // mask is position that exists opponent's stone from piece on each direction
        // left
        let mut mask = horizontal_watch & (self.player_board << 1);
        for _ in 0..5 {
            mask |= horizontal_watch & (mask << 1);
        }
        legal |= blank & (mask << 1);
        // right
        mask = horizontal_watch & (self.player_board >> 1);
        for _ in 0..5 {
            mask |= horizontal_watch & (mask >> 1);
        }
        legal |= blank & (mask >> 1);
        // up
        mask = vertical_watch & (self.player_board << 8);
        for _ in 0..5 {
            mask |= vertical_watch & (mask << 8);
        }
        legal |= blank & (mask << 8);
        // down
        mask = vertical_watch & (self.player_board >> 8);
        for _ in 0..5 {
            mask |= vertical_watch & (mask >> 8);
        }
        legal |= blank & (mask >> 8);
        // upper left
        mask = all_watch & (self.player_board << 9);
        for _ in 0..5 {
            mask |= all_watch & (mask << 9);
        }
        legal |= blank & (mask << 9);
        // upper right
        mask = all_watch & (self.player_board << 7);
        for _ in 0..5 {
            mask |= all_watch & (mask << 7);
        }
        legal |= blank & (mask << 7);
        // lower left
        mask = all_watch & (self.player_board >> 7);
        for _ in 0..5 {
            mask |= all_watch & (mask >> 7);
        }
        legal |= blank & (mask >> 7);
        // lower right
        mask = all_watch & (self.player_board >> 9);
        for _ in 0..5 {
            mask |= all_watch & (mask >> 9);
        }
        legal |= blank & (mask >> 9);
        legal
    }

    fn get_legal_moves_vec(&self) -> Vec<i32> {
        let legal_moves = self.get_legal_moves();
        let mut legal_moves_vec = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                if legal_moves & pos != 0 {
                    legal_moves_vec.push((i * BOARD_SIZE + j) as i32);
                }
            }
        }
        legal_moves_vec
    }

    fn is_legal_move(&self, pos: usize) -> bool {
        let pos = Board::pos2bit(pos);
        self.get_legal_moves() & pos != 0
    }

    fn reverse(&mut self, pos: u64) {
        let mut reversed: u64 = 0;
        let mut mask: u64;
        let mut tmp: u64;
        // mask is position that exists opponent's stone to reverse from piece on each direction
        // tmp is position of stones to reverse if piece exists on the end of stones to reverse
        // left
        const MASK_LEFT: u64 = 0xFE_FE_FE_FE_FE_FE_FE_FE;
        mask = MASK_LEFT & (pos << 1);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_LEFT & (mask << 1);
        }
        if (mask & self.player_board) != 0 {
            // if self.player_board exists on the end of stones to reverse
            reversed |= tmp;
        }
        // right
        const MASK_RIGHT: u64 = 0x7F_7F_7F_7F_7F_7F_7F_7F;
        mask = MASK_RIGHT & (pos >> 1);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_RIGHT & (mask >> 1);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // up
        const MASK_UP: u64 = 0xFF_FF_FF_FF_FF_FF_FF_00;
        mask = MASK_UP & (pos << 8);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_UP & (mask << 8);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // down
        const MASK_DOWN: u64 = 0x00_FF_FF_FF_FF_FF_FF_FF;
        mask = MASK_DOWN & (pos >> 8);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_DOWN & (mask >> 8);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // upper left
        const MASK_UPPER_LEFT: u64 = 0xFE_FE_FE_FE_FE_FE_FE_00;
        mask = MASK_UPPER_LEFT & (pos << 9);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_UPPER_LEFT & (mask << 9);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // upper right
        const MASK_UPPER_RIGHT: u64 = 0x7F_7F_7F_7F_7F_7F_7F_00;
        mask = MASK_UPPER_RIGHT & (pos << 7);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_UPPER_RIGHT & (mask << 7);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // lower left
        const MASK_LOWER_LEFT: u64 = 0xFE_FE_FE_FE_FE_FE_FE;
        mask = MASK_LOWER_LEFT & (pos >> 7);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_LOWER_LEFT & (mask >> 7);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        // lower right
        const MASK_LOWER_RIGHT: u64 = 0x7F_7F_7F_7F_7F_7F_7F;
        mask = MASK_LOWER_RIGHT & (pos >> 9);
        tmp = 0;
        while mask & self.opponent_board != 0 {
            tmp |= mask;
            mask = MASK_LOWER_RIGHT & (mask >> 9);
        }
        if (mask & self.player_board) != 0 {
            reversed |= tmp;
        }
        self.player_board ^= reversed | pos;
        self.opponent_board ^= reversed;
    }

    fn do_move(&mut self, pos: usize) -> PyResult<()> {
        if pos >= BOARD_SIZE * BOARD_SIZE {
            return Err(PyValueError::new_err("Invalid position"));
        }
        let pos_bit = Board::pos2bit(pos);
        if self.is_legal_move(pos) {
            self.reverse(pos_bit);
            swap(&mut self.player_board, &mut self.opponent_board);
            self.turn = self.turn.opposite();
        } else {
            return Err(PyValueError::new_err("Invalid move"));
        }
        Ok(())
    }

    fn do_pass(&mut self) -> PyResult<()> {
        if self.get_legal_moves() == 0 {
            swap(&mut self.player_board, &mut self.opponent_board);
            self.turn = self.turn.opposite();
        } else {
            return Err(PyValueError::new_err("Invalid pass"));
        }
        Ok(())
    }

    fn is_pass(&self) -> bool {
        self.get_legal_moves() == 0
    }

    fn is_game_over(&self) -> bool {
        let opponent_board = Board {
            player_board: self.opponent_board,
            opponent_board: self.player_board,
            turn: self.turn.opposite(),
        };
        self.is_pass() && opponent_board.is_pass()
    }

    fn is_win(&self) -> PyResult<bool> {
        if self.is_game_over() {
            Ok(self.player_piece_num() > self.opponent_piece_num())
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }

    fn is_lose(&self) -> PyResult<bool> {
        if self.is_game_over() {
            Ok(self.player_piece_num() < self.opponent_piece_num())
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }

    fn is_draw(&self) -> PyResult<bool> {
        if self.is_game_over() {
            Ok(self.player_piece_num() == self.opponent_piece_num())
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }

    fn is_black_win(&self) -> PyResult<bool> {
        if self.is_game_over() {
            Ok(self.black_piece_num() > self.white_piece_num())
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }

    fn is_white_win(&self) -> PyResult<bool> {
        if self.is_game_over() {
            Ok(self.white_piece_num() > self.black_piece_num())
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }

    fn get_winner(&self) -> PyResult<Option<Turn>> {
        if self.is_game_over() {
            if self.is_win().unwrap() {
                return Ok(Some(self.turn));
            } else if self.is_lose().unwrap() {
                return match self.turn {
                    Turn::Black => Ok(Some(Turn::White)),
                    Turn::White => Ok(Some(Turn::Black)),
                };
            } else {
                return Ok(None);
            }
        } else {
            Err(PyValueError::new_err("Game is not over yet"))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_reversi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Turn>()?;
    m.add_class::<Color>()?;
    m.add_class::<Board>()?;
    Ok(())
}
