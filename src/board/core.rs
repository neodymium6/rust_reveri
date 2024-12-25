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
    pub fn opposite(&self) -> Turn {
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

#[derive(Clone, PartialEq)]
pub struct Board {
    player_board: u64,
    opponent_board: u64,
    turn: Turn,
}

const BITS: [u64; 64] = [
    1 << 63,
    1 << 62,
    1 << 61,
    1 << 60,
    1 << 59,
    1 << 58,
    1 << 57,
    1 << 56,
    1 << 55,
    1 << 54,
    1 << 53,
    1 << 52,
    1 << 51,
    1 << 50,
    1 << 49,
    1 << 48,
    1 << 47,
    1 << 46,
    1 << 45,
    1 << 44,
    1 << 43,
    1 << 42,
    1 << 41,
    1 << 40,
    1 << 39,
    1 << 38,
    1 << 37,
    1 << 36,
    1 << 35,
    1 << 34,
    1 << 33,
    1 << 32,
    1 << 31,
    1 << 30,
    1 << 29,
    1 << 28,
    1 << 27,
    1 << 26,
    1 << 25,
    1 << 24,
    1 << 23,
    1 << 22,
    1 << 21,
    1 << 20,
    1 << 19,
    1 << 18,
    1 << 17,
    1 << 16,
    1 << 15,
    1 << 14,
    1 << 13,
    1 << 12,
    1 << 11,
    1 << 10,
    1 << 9,
    1 << 8,
    1 << 7,
    1 << 6,
    1 << 5,
    1 << 4,
    1 << 3,
    1 << 2,
    1 << 1,
    1 << 0,
];

impl Default for Board {
    fn default() -> Self {
        Board {
            player_board: 0x00_00_00_08_10_00_00_00,
            opponent_board: 0x00_00_00_10_08_00_00_00,
            turn: Turn::Black,
        }
    }
}

impl Board {
    pub fn new() -> Board {
        Board::default()
    }

    fn pos2bit(pos: usize) -> u64 {
        BITS[pos]
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

    pub fn get_board_line(&self) -> Result<String, BoardError> {
        let mut board_str = String::new();
        let player_char = match self.turn {
            Turn::Black => LINE_CHAR_BLACK,
            Turn::White => LINE_CHAR_WHITE,
        };
        let opponent_char = match self.turn {
            Turn::Black => LINE_CHAR_WHITE,
            Turn::White => LINE_CHAR_BLACK,
        };
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            let pos = Board::pos2bit(i);
            match (self.player_board & pos, self.opponent_board & pos) {
                (0, 0) => board_str.push(LINE_CHAR_EMPTY),
                (_, 0) => board_str.push(player_char),
                (0, _) => board_str.push(opponent_char),
                (_, _) => return Err(BoardError::InvalidState),
            }
        }
        Ok(board_str)
    }

    pub fn get_board_vec_black(&self) -> Result<Vec<Color>, BoardError> {
        let mut board_vec = vec![Color::Empty; BOARD_SIZE * BOARD_SIZE];
        for (i, board_vec_elem) in board_vec.iter_mut().enumerate() {
            let bit = Board::pos2bit(i);
            *board_vec_elem = match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => Color::Empty,
                (_, 0) => Color::Black,
                (0, _) => Color::White,
                (_, _) => return Err(BoardError::InvalidState),
            };
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
        for (i, board_vec_elem) in board_vec.iter_mut().enumerate() {
            let bit = Board::pos2bit(i);
            *board_vec_elem = match (self.player_board & bit, self.opponent_board & bit) {
                (0, 0) => Color::Empty,
                (_, 0) => player_color,
                (0, _) => opponent_color,
                (_, _) => return Err(BoardError::InvalidState),
            };
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

    fn get_legal_partial(watch: u64, player_board: u64, shift: usize) -> u64 {
        let mut flip_l = (player_board << shift) & watch;
        let mut flip_r = (player_board >> shift) & watch;
        flip_l |= (flip_l << shift) & watch;
        flip_r |= (flip_r >> shift) & watch;
        let watch_l = watch & (watch << shift);
        let watch_r = watch & (watch >> shift);
        let shift2 = shift + shift;
        flip_l |= (flip_l << shift2) & watch_l;
        flip_r |= (flip_r >> shift2) & watch_r;
        flip_l |= (flip_l << shift2) & watch_l;
        flip_r |= (flip_r >> shift2) & watch_r;
        flip_l << shift | flip_r >> shift
    }

    pub fn get_legal_moves(&self) -> u64 {
        let mask = 0x7E_7E_7E_7E_7E_7E_7E_7E & self.opponent_board;
        (Board::get_legal_partial(mask, self.player_board, 1)
            | Board::get_legal_partial(self.opponent_board, self.player_board, 8)
            | Board::get_legal_partial(mask, self.player_board, 9)
            | Board::get_legal_partial(mask, self.player_board, 7))
            & !(self.player_board | self.opponent_board)
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

    pub fn get_legal_moves_tf(&self) -> Vec<bool> {
        let legal_moves = self.get_legal_moves();
        let mut legal_moves_tf = Vec::new();
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            legal_moves_tf.push(legal_moves & Board::pos2bit(i) != 0);
        }
        legal_moves_tf
    }

    pub fn is_legal_move(&self, pos: usize) -> bool {
        self.get_legal_moves() & Board::pos2bit(pos) != 0
    }

    pub fn get_child_boards(&self) -> Option<Vec<Board>> {
        if self.is_pass() {
            return None;
        }
        let legal_moves_vec = self.get_legal_moves_vec();
        let mut child_boards = Vec::new();
        for pos in legal_moves_vec {
            let mut child_board = self.clone();
            child_board.do_move(pos).unwrap();
            child_boards.push(child_board);
        }
        Some(child_boards)
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
        let mask_v = 0x7E_7E_7E_7E_7E_7E_7E_7E & self.opponent_board;
        let mask_h = 0x00_FF_FF_FF_FF_FF_FF_00 & self.opponent_board;
        let mask_a = 0x00_7E_7E_7E_7E_7E_7E_00 & self.opponent_board;
        let enmpy = !(self.player_board | self.opponent_board);
        if Board::get_legal_partial(mask_v, self.player_board, 1) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_h, self.player_board, 8) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_a, self.player_board, 9) & enmpy != 0 {
            return false;
        }
        if Board::get_legal_partial(mask_a, self.player_board, 7) & enmpy != 0 {
            return false;
        }
        true
    }

    pub fn is_game_over(&self) -> bool {
        if self.is_pass() {
            let opponent_board = Board {
                player_board: self.opponent_board,
                opponent_board: self.player_board,
                turn: self.turn.opposite(),
            };
            if opponent_board.is_pass() {
                return true;
            }
        }
        false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_1000000games() {
        for _ in 0..1000000 {
            let mut board = Board::new();
            while !board.is_game_over() {
                if board.is_pass() {
                    board.do_pass().unwrap();
                } else {
                    let pos = board.get_random_move().unwrap();
                    board.do_move(pos).unwrap();
                }
            }
        }
    }
}
