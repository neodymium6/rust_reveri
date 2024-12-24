use pyo3::prelude::*;

mod alpha_beta;
mod evaluator;

use crate::board::Board;
use alpha_beta::AlphaBetaSearch as RustAlphaBetaSearch;
use evaluator::{Evaluator as RustEvaluator, PieceEvaluator as RustPieceEvaluator, LegalNumEvaluator as RustLegalNumEvaluator};

#[derive(Clone)]
enum EvaluatorType {
    Piece(RustPieceEvaluator),
    LegalNum(RustLegalNumEvaluator),
}

impl EvaluatorType {
    fn as_evaluator(&self) -> &dyn RustEvaluator {
        match self {
            EvaluatorType::Piece(e) => e,
            EvaluatorType::LegalNum(e) => e,
        }
    }

    fn as_evaluator_box(&self) -> Box<dyn RustEvaluator> {
        match self {
            EvaluatorType::Piece(e) => Box::new(e.clone()),
            EvaluatorType::LegalNum(e) => Box::new(e.clone()),
        }
    }
}

#[pyclass(subclass)]
#[derive(Clone)]
pub struct Evaluator {
    inner: EvaluatorType,
}

#[pymethods]
impl Evaluator {
    #[new]
    fn new() -> Self {
        Evaluator {
            inner: EvaluatorType::Piece(RustPieceEvaluator {}),
        }
    }

    fn evaluate(&self, board: &Board) -> i32 {
        self.inner.as_evaluator().evaluate(&board.inner)
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


#[pyclass]
pub struct AlphaBetaSearch {
    inner: RustAlphaBetaSearch,
}

#[pymethods]
impl AlphaBetaSearch {
    #[new]
    fn new(evaluator: Evaluator, max_depth: usize) -> Self {
        let rust_evaluator = evaluator.inner;
        AlphaBetaSearch {
            inner: RustAlphaBetaSearch::new(max_depth, rust_evaluator.as_evaluator_box()),
        }
    }

    fn get_move(&self, board: Board) -> Option<usize> {
        self.inner.get_move(board.inner)
    }
}
