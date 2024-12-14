use pyo3::{exceptions::PyValueError, prelude::*};

mod board;
use board::{Turn as RustTurn, Color as RustColor, Board as RustBoard, BoardError};



#[pyclass(eq)]
#[derive(Clone, PartialEq)]
struct Turn {
    inner: RustTurn,
}

#[pymethods]
impl Turn {
    #[classattr]
    const BLACK: Self = Turn { inner: RustTurn::Black };
    #[classattr]
    const WHITE: Self = Turn { inner: RustTurn::White };

    fn __str__(&self) -> &'static str {
        match self.inner {
            RustTurn::Black => "Black",
            RustTurn::White => "White",
        }
    }
}

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
struct Color {
    inner: RustColor,
}

#[pymethods]
impl Color {
    #[classattr]
    const EMPTY: Self = Color { inner: RustColor::Empty };
    #[classattr]
    const BLACK: Self = Color { inner: RustColor::Black };
    #[classattr]
    const WHITE: Self = Color { inner: RustColor::White };

    fn __str__(&self) -> &'static str {
        match self.inner {
            RustColor::Empty => "Empty",
            RustColor::Black => "Black",
            RustColor::White => "White",
        }
    }
}

#[pyclass]
struct Board {
    inner: RustBoard,
}

#[pymethods]
impl Board {
    #[new]
    fn new() -> Self {
        Board {
            inner: RustBoard::new(),
        }
    }

    fn get_board(&self) -> (u64, u64, Turn) {
        let player_board: u64;
        let opponent_board: u64;
        let turn: RustTurn;
        (player_board, opponent_board, turn) = self.inner.get_board();
        (player_board, opponent_board, Turn { inner: turn })
    }

    fn set_board(&mut self, player_board: u64, opponent_board: u64, turn: Turn) {
        self.inner.set_board(player_board, opponent_board, turn.inner);
    }

    fn set_board_str(&mut self, line: &str, turn: Turn) -> PyResult<()> {
        self.inner.set_board_str(line, turn.inner).map_err(|e| match e {
            BoardError::InvalidCharactor => PyValueError::new_err("Invalid charactor"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn get_board_vec_black(&self) -> PyResult<Vec<Color>> {
        self.inner.get_board_vec_black()
            .map(|vec| {
                vec.into_iter()
                    .map(|color| Color { inner: color })
                    .collect()
            })
            .map_err(|e| match e {
                BoardError::InvalidState => PyValueError::new_err("Invalid state"),
                _ => PyValueError::new_err("Unexpected error"),
            })
    }

    fn get_board_vec_turn(&self) -> PyResult<Vec<Color>> {
        self.inner.get_board_vec_turn()
            .map(|vec| {
                vec.into_iter()
                    .map(|color| Color { inner: color })
                    .collect()
            })
            .map_err(|e| match e {
                BoardError::InvalidState => PyValueError::new_err("Invalid state"),
                _ => PyValueError::new_err("Unexpected error"),
            })
    }

    fn get_board_matrix(&self) -> PyResult<Vec<Vec<Vec<i32>>>> {
        self.inner.get_board_matrix()
            .map_err(|e| match e {
                BoardError::InvalidState => PyValueError::new_err("Invalid state"),
                _ => PyValueError::new_err("Unexpected error"),
            })
    }

    fn player_piece_num(&self) -> i32 {
        self.inner.player_piece_num()
    }

    fn opponent_piece_num(&self) -> i32 {
        self.inner.opponent_piece_num()
    }

    fn black_piece_num(&self) -> i32 {
        self.inner.black_piece_num()
    }

    fn white_piece_num(&self) -> i32 {
        self.inner.white_piece_num()
    }

    fn piece_sum(&self) -> i32 {
        self.inner.piece_sum()
    }

    fn diff_piece_num(&self) -> i32 {
        self.inner.diff_piece_num()
    }

    fn get_legal_moves(&self) -> u64 {
        self.inner.get_legal_moves()
    }

    fn get_legal_moves_vec(&self) -> Vec<usize> {
        self.inner.get_legal_moves_vec()
    }

    fn is_legal_move(&self, pos: usize) -> bool {
        self.inner.is_legal_move(pos)
    }

    fn reverse(&mut self, pos: u64) {
        self.inner.reverse(pos)
    }

    fn do_move(&mut self, pos: usize) -> PyResult<()> {
        self.inner.do_move(pos).map_err(|e| match e {
            BoardError::InvalidPosition => PyValueError::new_err("Invalid position"),
            BoardError::InvalidMove => PyValueError::new_err("Invalid move"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn do_pass(&mut self) -> PyResult<()> {
        self.inner.do_pass().map_err(|e| match e {
            BoardError::InvalidPass => PyValueError::new_err("Invalid pass"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn is_pass(&self) -> bool {
        self.inner.is_pass()
    }

    fn is_game_over(&self) -> bool {
        self.inner.is_game_over()
    }

    fn is_win(&self) -> PyResult<bool> {
        self.inner.is_win().map_err(|e| match e {
            BoardError::GameNotOverYet => PyValueError::new_err("Game is not over yet"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn is_lose(&self) -> PyResult<bool> {
        self.inner.is_lose().map_err(|e| match e {
            BoardError::GameNotOverYet => PyValueError::new_err("Game is not over yet"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn is_draw(&self) -> PyResult<bool> {
        self.inner.is_draw().map_err(|e| match e {
            BoardError::GameNotOverYet => PyValueError::new_err("Game is not over yet"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn is_black_win(&self) -> PyResult<bool> {
        self.inner.is_black_win().map_err(|e| match e {
            BoardError::GameNotOverYet => PyValueError::new_err("Game is not over yet"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn is_white_win(&self) -> PyResult<bool> {
        self.inner.is_white_win().map_err(|e| match e {
            BoardError::GameNotOverYet => PyValueError::new_err("Game is not over yet"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn get_winner(&self) -> PyResult<Option<Turn>> {
        match self.inner.get_winner() {
            Ok(Some(turn)) => Ok(Some(Turn { inner: turn })),
            Ok(None) => Ok(None),
            Err(e) => match e {
                BoardError::GameNotOverYet => Err(PyValueError::new_err("Game is not over yet")),
                _ => Err(PyValueError::new_err("Unexpected error")),
            },
        }
    }

    fn get_random_move(&self) -> PyResult<usize> {
        self.inner.get_random_move().map_err(|e| match e {
            BoardError::NoLegalMove => PyValueError::new_err("No legal move"),
            _ => PyValueError::new_err("Unexpected error"),
        })
    }

    fn __str__(&self) -> PyResult<String> {
        self.inner.to_string()
            .map_err(|e| match e {
                BoardError::InvalidState => PyValueError::new_err("Invalid state"),
                _ => PyValueError::new_err("Unexpected error"),
            })
    }

    fn clone(&self) -> Self {
        Board {
            inner: self.inner.clone(),
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
