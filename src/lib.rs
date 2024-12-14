use pyo3::prelude::*;

mod board;
use board::{Turn, Color, Board};

mod arena;
use arena::Arena;

#[pymodule]
fn rust_reversi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Turn>()?;
    m.add_class::<Color>()?;
    m.add_class::<Board>()?;
    m.add_class::<Arena>()?;
    Ok(())
}
