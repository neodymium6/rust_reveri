use pyo3::prelude::*;

mod board;
use board::{Board, Color, Turn};

mod arena;
use arena::{Arena, NetworkArenaClient, NetworkArenaServer};

mod search;
use search::{AlphaBetaSearch, Evaluator, LegalNumEvaluator, MatrixEvaluator, PieceEvaluator};

#[pymodule]
fn rust_reversi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Turn>()?;
    m.add_class::<Color>()?;
    m.add_class::<Board>()?;
    m.add_class::<Arena>()?;
    m.add_class::<NetworkArenaClient>()?;
    m.add_class::<NetworkArenaServer>()?;
    m.add_class::<AlphaBetaSearch>()?;
    m.add_class::<LegalNumEvaluator>()?;
    m.add_class::<MatrixEvaluator>()?;
    m.add_class::<PieceEvaluator>()?;
    m.add_class::<Evaluator>()?;
    Ok(())
}
