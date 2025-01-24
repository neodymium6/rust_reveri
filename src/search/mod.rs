use std::sync::Arc;

use pyo3::prelude::*;

use crate::board::Board;
use rust_reversi_core::board::Board as RustBoard;
use rust_reversi_core::search::{
    AlphaBetaSearch as RustAlphaBetaSearch, Search, ThunderSearch as RustThunderSearch,
};
use rust_reversi_core::search::{
    BitMatrixEvaluator as RustBitMatrixEvaluator, Evaluator as RustEvaluator,
    LegalNumEvaluator as RustLegalNumEvaluator, MatrixEvaluator as RustMatrixEvaluator,
    PieceEvaluator as RustPieceEvaluator, WinrateEvaluator as RustWinrateEvaluator,
};

#[derive(Clone, Debug)]
struct PyEvaluator {
    py_evaluator: Arc<Py<PyAny>>,
}

impl RustEvaluator for PyEvaluator {
    fn evaluate(&self, board: &mut RustBoard) -> i32 {
        Python::with_gil(|py| {
            let board_wrapper = Board {
                inner: board.clone(),
            };
            let result = self
                .py_evaluator
                .call_method1(py, "evaluate", (board_wrapper,))
                .expect("Failed to call evaluate method");
            result.extract(py).expect("Failed to extract result")
        })
    }
}

#[derive(Clone)]
enum EvaluatorType {
    Piece(RustPieceEvaluator),
    LegalNum(RustLegalNumEvaluator),
    Matrix(Arc<RustMatrixEvaluator>),
    Python(PyEvaluator),
}

impl EvaluatorType {
    fn as_evaluator(&self) -> Arc<dyn RustEvaluator> {
        match self {
            EvaluatorType::Piece(e) => Arc::new(e.clone()),
            EvaluatorType::LegalNum(e) => Arc::new(e.clone()),
            EvaluatorType::Matrix(e) => e.clone(),
            EvaluatorType::Python(e) => Arc::new(e.clone()),
        }
    }
}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct Evaluator {
    inner: EvaluatorType,
}

impl Default for Evaluator {
    fn default() -> Self {
        Evaluator {
            inner: EvaluatorType::Piece(RustPieceEvaluator {}),
        }
    }
}

#[pymethods]
impl Evaluator {
    #[new]
    fn new() -> Self {
        Evaluator::default()
    }

    fn set_py_evaluator(&mut self, py_evaluator: Py<PyAny>) {
        self.inner = EvaluatorType::Python(PyEvaluator {
            py_evaluator: Arc::new(py_evaluator),
        });
    }

    fn evaluate(&self, board: &mut Board) -> i32 {
        self.inner.as_evaluator().evaluate(&mut board.inner)
    }
}

#[pyclass(extends=Evaluator)]
#[derive(Clone)]
pub struct PieceEvaluator {}

#[pymethods]
impl PieceEvaluator {
    #[new]
    fn new() -> (Self, Evaluator) {
        let evaluator = Evaluator {
            inner: EvaluatorType::Piece(RustPieceEvaluator {}),
        };
        (PieceEvaluator {}, evaluator)
    }
}

#[pyclass(extends=Evaluator)]
pub struct LegalNumEvaluator {}

#[pymethods]
impl LegalNumEvaluator {
    #[new]
    fn new() -> (Self, Evaluator) {
        let evaluator = Evaluator {
            inner: EvaluatorType::LegalNum(RustLegalNumEvaluator {}),
        };
        (LegalNumEvaluator {}, evaluator)
    }
}

#[pyclass(extends=Evaluator)]
pub struct MatrixEvaluator {}

#[pymethods]
impl MatrixEvaluator {
    #[new]
    fn new(matrix: [[i32; 8]; 8]) -> (Self, Evaluator) {
        let evaluator = Evaluator {
            inner: EvaluatorType::Matrix(Arc::new(RustMatrixEvaluator::new(matrix))),
        };
        (MatrixEvaluator {}, evaluator)
    }
}

#[pyclass]
pub struct AlphaBetaSearch {
    inner: RustAlphaBetaSearch,
}

#[pymethods]
impl AlphaBetaSearch {
    #[new]
    fn new(evaluator: Evaluator, max_depth: usize, win_score: i32) -> Self {
        let rust_evaluator = evaluator.inner;
        AlphaBetaSearch {
            inner: RustAlphaBetaSearch::new(max_depth, rust_evaluator.as_evaluator(), win_score),
        }
    }

    fn get_move(&self, board: &mut Board) -> Option<usize> {
        self.inner.get_move(&mut board.inner)
    }

    fn get_move_with_timeout(&self, board: &mut Board, timeout_ms: u64) -> Option<usize> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        self.inner.get_move_with_timeout(&mut board.inner, timeout)
    }

    fn get_search_score(&self, board: &mut Board) -> f64 {
        self.inner.get_search_score(&mut board.inner)
    }
}

#[derive(Clone, Debug)]
struct PyWinrateEvaluator {
    py_evaluator: Arc<Py<PyAny>>,
}

impl RustWinrateEvaluator for PyWinrateEvaluator {
    fn evaluate(&self, board: &mut RustBoard) -> f64 {
        Python::with_gil(|py| {
            let board_wrapper = Board {
                inner: board.clone(),
            };
            let result = self
                .py_evaluator
                .call_method1(py, "evaluate", (board_wrapper,))
                .expect("Failed to call evaluate method");
            result.extract(py).expect("Failed to extract result")
        })
    }
}

#[derive(Debug, Clone)]
struct BMWinEvaluator {
    evaluator: RustBitMatrixEvaluator<10>,
}
impl BMWinEvaluator {
    fn new() -> BMWinEvaluator {
        let masks: Vec<u64> = vec![
            0x0000001818000000,
            0x0000182424180000,
            0x0000240000240000,
            0x0018004242001800,
            0x0024420000422400,
            0x0042000000004200,
            0x1800008181000018,
            0x2400810000810024,
            0x4281000000008142,
            0x8100000000000081,
        ];
        let weights: Vec<i32> = vec![0, 0, -1, -6, -8, -12, 0, 4, 1, 40];
        let evaluator = RustBitMatrixEvaluator::<10>::new(weights, masks);
        BMWinEvaluator { evaluator }
    }
}
impl RustWinrateEvaluator for BMWinEvaluator {
    fn evaluate(&self, board: &mut RustBoard) -> f64 {
        let v = self.evaluator.evaluate(board) as f64;
        let max = 300.0;
        (v + max) / (2.0 * max)
    }
}

#[derive(Clone)]
enum WinrateEvaluatorType {
    BitMatrix(BMWinEvaluator),
    Python(PyWinrateEvaluator),
}

impl WinrateEvaluatorType {
    fn as_evaluator(&self) -> Arc<dyn RustWinrateEvaluator> {
        match self {
            WinrateEvaluatorType::BitMatrix(e) => Arc::new(e.clone()),
            WinrateEvaluatorType::Python(e) => Arc::new(e.clone()),
        }
    }
}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct WinrateEvaluator {
    inner: WinrateEvaluatorType,
}

impl Default for WinrateEvaluator {
    fn default() -> Self {
        WinrateEvaluator {
            inner: WinrateEvaluatorType::BitMatrix(BMWinEvaluator::new()),
        }
    }
}

#[pymethods]
impl WinrateEvaluator {
    #[new]
    fn new() -> Self {
        WinrateEvaluator::default()
    }

    fn set_py_evaluator(&mut self, py_evaluator: Py<PyAny>) {
        self.inner = WinrateEvaluatorType::Python(PyWinrateEvaluator {
            py_evaluator: Arc::new(py_evaluator),
        });
    }

    fn evaluate(&self, board: &mut Board) -> f64 {
        self.inner.as_evaluator().evaluate(&mut board.inner)
    }
}

#[pyclass]
pub struct ThunderSearch {
    inner: RustThunderSearch,
}

#[pymethods]
impl ThunderSearch {
    #[new]
    fn new(evaluator: WinrateEvaluator, n_playouts: usize, epsilon: f64) -> Self {
        let rust_evaluator = evaluator.inner;
        ThunderSearch {
            inner: RustThunderSearch::new(n_playouts, epsilon, rust_evaluator.as_evaluator()),
        }
    }

    fn get_move(&self, board: &mut Board) -> Option<usize> {
        self.inner.get_move(&mut board.inner)
    }

    fn get_move_with_timeout(&self, board: &mut Board, timeout_ms: u64) -> Option<usize> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        self.inner.get_move_with_timeout(&mut board.inner, timeout)
    }

    fn get_search_score(&self, board: &mut Board) -> f64 {
        self.inner.get_search_score(&mut board.inner)
    }
}
