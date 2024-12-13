use pyo3::prelude::*;

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

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
enum Color {
    Empty,
    Black,
    White,
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

    fn set_board_str(&mut self, line: &str, turn: Turn) {
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
                _ => panic!("Invalid line character"),
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
    }

    fn get_board_vec(&self) -> Vec<i32> {
        let mut board_vec = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                match (self.player_board & pos == 0, self.opponent_board & pos == 0) {
                    (true, true) => board_vec.push(Color::Empty as i32),    // Empty
                    (false, true) => board_vec.push(Color::Black as i32),   // Player
                    (true, false) => board_vec.push(Color::White as i32),   // Opponent
                    _ => panic!("Invalid board state"),
                }
            }
        }
        board_vec
    }

    fn get_board_matrix(&self) -> Vec<Vec<Vec<i32>>> {
        let mut board_matrix = vec![vec![vec![0; BOARD_SIZE]; BOARD_SIZE]; 3];
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = Board::pos2bit(i * BOARD_SIZE + j);
                match (self.player_board & pos == 0, self.opponent_board & pos == 0) {
                    (true, true) => board_matrix[2][i][j] = 1,  // Empty
                    (false, true) => board_matrix[0][i][j] = 1, // Player
                    (true, false) => board_matrix[1][i][j] = 1, // Opponent
                    _ => panic!("Invalid board state"),
                }
            }
        }
        board_matrix
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
