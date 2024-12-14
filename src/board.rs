use core::fmt;
use std::mem::swap;

const BOARD_SIZE: usize = 8;
const LINE_CHAR_BLACK: char = 'X';
const LINE_CHAR_WHITE: char = 'O';
const LINE_CHAR_EMPTY: char = '-';

#[derive(Debug)]
pub enum BoardError {
    InvalidPosition,
    InvalidMove,
    InvalidPass,
    InvalidState,
    GameNotOverYet,
    InvalidCharactor,
    NoLegalMove,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Turn {
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

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    Empty,
    Black,
    White,
}

impl Color {
    fn opposite(&self) -> Color {
        match self {
            Color::Empty => Color::Empty,
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Board {
    player_board: u64,
    opponent_board: u64,
    turn: Turn,
}

impl Board {
    pub fn new() -> Board {
        Board {
            player_board: 0x00_00_00_08_10_00_00_00,
            opponent_board: 0x00_00_00_10_08_00_00_00,
            turn: Turn::Black,
        }
    }

    fn pos2bit(pos: usize) -> u64 {
        1 << (BOARD_SIZE * BOARD_SIZE - pos - 1)
    }

    pub fn get_board(&self) -> (u64, u64, Turn) {
        (self.player_board, self.opponent_board, self.turn)
    }

    pub fn set_board(&mut self, player_board: u64, opponent_board: u64, turn: Turn) {
        self.player_board = player_board;
        self.opponent_board = opponent_board;
        self.turn = turn;
    }

    pub fn set_board_str(&mut self, board_str: &str, turn: Turn) -> Result<(), BoardError> {
        let mut black_board = 0;
        let mut white_board = 0;
        for (i, c) in board_str.chars().enumerate() {
            match c {
                LINE_CHAR_BLACK => black_board |= Board::pos2bit(i),
                LINE_CHAR_WHITE => white_board |= Board::pos2bit(i),
                LINE_CHAR_EMPTY => (),
                _ => {
                    return Err(BoardError::InvalidCharactor);
                }
            }
        }
        match turn {
            Turn::Black => self.set_board(black_board, white_board, Turn::Black),
            Turn::White => self.set_board(white_board, black_board, Turn::White),
        }
        Ok(())
    }

    pub fn get_board_vec_black(&self) -> Result<Vec<Color>, BoardError> {
        let mut board_vec = vec![Color::Empty; BOARD_SIZE * BOARD_SIZE];
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            let bit = Board::pos2bit(i);
            match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => board_vec[i] = Color::Empty,
                (_, 0) => board_vec[i] = Color::Black,
                (0, _) => board_vec[i] = Color::White,
                (_, _) => return Err(BoardError::InvalidState),
            }
        }
        Ok(board_vec)
    }

    pub fn get_board_vec_turn(&self) -> Result<Vec<Color>, BoardError> {
        let mut board_vec = vec![Color::Empty; BOARD_SIZE * BOARD_SIZE];
        let player_color = match self.turn {
            Turn::Black => Color::Black,
            Turn::White => Color::White,
        };
        let opponent_color = player_color.opposite();
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            let bit = Board::pos2bit(i);
            match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => board_vec[i] = Color::Empty,
                (_, 0) => board_vec[i] = player_color,
                (0, _) => board_vec[i] = opponent_color,
                (_, _) => return Err(BoardError::InvalidState),
            }
        }
        Ok(board_vec)
    }

    pub fn get_board_matrix(&self) -> Result<Vec<Vec<Vec<i32>>>, BoardError> {
        let mut board_matrix = vec![vec![vec![0; BOARD_SIZE]; BOARD_SIZE]; 3];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let i = x * BOARD_SIZE + y;
                let bit = Board::pos2bit(i);
                match (self.player_board & bit, self.opponent_board & bit) {
                    (0, 0) => board_matrix[2][x][y] = 1,
                    (_, 0) => board_matrix[0][x][y] = 1,
                    (0, _) => board_matrix[1][x][y] = 1,
                    (_, _) => return Err(BoardError::InvalidState),
                }
            }
        }
        Ok(board_matrix)
    }

    pub fn player_piece_num(&self) -> i32 {
        self.player_board.count_ones() as i32
    }

    pub fn opponent_piece_num(&self) -> i32 {
        self.opponent_board.count_ones() as i32
    }

    pub fn black_piece_num(&self) -> i32 {
        if self.turn == Turn::Black {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    pub fn white_piece_num(&self) -> i32 {
        if self.turn == Turn::White {
            self.player_piece_num()
        } else {
            self.opponent_piece_num()
        }
    }

    pub fn piece_sum(&self) -> i32 {
        self.player_piece_num() + self.opponent_piece_num()
    }

    pub fn diff_piece_num(&self) -> i32 {
        self.player_piece_num() - self.opponent_piece_num()
    }

    pub fn get_legal_moves(&self) -> u64 {
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

    pub fn get_legal_moves_vec(&self) -> Vec<usize> {
        let legal_moves = self.get_legal_moves();
        let mut legal_moves_vec = Vec::new();
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            if legal_moves & Board::pos2bit(i) != 0 {
                legal_moves_vec.push(i);
            }
        }
        legal_moves_vec
    }

    pub fn is_legal_move(&self, pos: usize) -> bool {
        self.get_legal_moves() & Board::pos2bit(pos) != 0
    }

    pub fn reverse(&mut self, pos: u64) {
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

    pub fn do_move(&mut self, pos: usize) -> Result<(), BoardError> {
        if pos >= BOARD_SIZE * BOARD_SIZE {
            return Err(BoardError::InvalidPosition);
        }
        let pos_bit = Board::pos2bit(pos);
        if self.is_legal_move(pos) {
            self.reverse(pos_bit);
            swap(&mut self.player_board, &mut self.opponent_board);
            self.turn = self.turn.opposite();
        } else {
            return Err(BoardError::InvalidMove);
        }
        Ok(())
    }

    pub fn do_pass(&mut self) -> Result<(), BoardError> {
        if self.get_legal_moves() == 0 {
            swap(&mut self.player_board, &mut self.opponent_board);
            self.turn = self.turn.opposite();
        } else {
            return Err(BoardError::InvalidPass);
        }
        Ok(())
    }

    pub fn is_pass(&self) -> bool {
        self.get_legal_moves() == 0
    }

    pub fn is_game_over(&self) -> bool {
        let opponent_board = Board {
            player_board: self.opponent_board,
            opponent_board: self.player_board,
            turn: self.turn.opposite(),
        };
        self.is_pass() && opponent_board.is_pass()
    }

    pub fn is_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() > self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn is_lose(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() < self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn is_draw(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.player_piece_num() == self.opponent_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn is_black_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.black_piece_num() > self.white_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn is_white_win(&self) -> Result<bool, BoardError> {
        if self.is_game_over() {
            Ok(self.white_piece_num() > self.black_piece_num())
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn get_winner(&self) -> Result<Option<Turn>, BoardError> {
        if self.is_game_over() {
            if self.is_win().unwrap() {
                Ok(Some(self.turn))
            } else if self.is_lose().unwrap() {
                Ok(Some(self.turn.opposite()))
            } else {
                Ok(None)
            }
        } else {
            Err(BoardError::GameNotOverYet)
        }
    }

    pub fn get_random_move(&self) -> Result<usize, BoardError> {
        let legal_moves_vec = self.get_legal_moves_vec();
        if legal_moves_vec.is_empty() {
            return Err(BoardError::NoLegalMove);
        }
        let random_index = rand::random::<usize>() % legal_moves_vec.len();
        Ok(legal_moves_vec[random_index])
    }

    pub fn to_string(&self) -> Result<String, BoardError> {
        let mut board_str = String::new();
        let player_char = match self.turn {
            Turn::Black => LINE_CHAR_BLACK,
            Turn::White => LINE_CHAR_WHITE,
        };
        let opponent_char = match self.turn {
            Turn::Black => LINE_CHAR_WHITE,
            Turn::White => LINE_CHAR_BLACK,
        };
        board_str.push_str(" |abcdefgh\n-+--------\n");
        for i in 0..BOARD_SIZE {
            board_str.push_str(&format!("{}|", i + 1));
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                match (self.player_board & pos, self.opponent_board & pos) {
                    (0, 0) => board_str.push(LINE_CHAR_EMPTY),
                    (_, 0) => board_str.push(player_char),
                    (0, _) => board_str.push(opponent_char),
                    (_, _) => return Err(BoardError::InvalidState),
                }
            }
            board_str.push('\n');
        }
        Ok(board_str)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string().unwrap())
    }
    
}