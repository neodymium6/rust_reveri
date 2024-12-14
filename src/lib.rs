use pyo3::prelude::*;

mod board;
use board::{Turn, Color, Board};

/// A Python module implemented in Rust.
#[pymodule]
fn rust_reversi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Turn>()?;
    m.add_class::<Color>()?;
    m.add_class::<Board>()?;
    Ok(())
}
