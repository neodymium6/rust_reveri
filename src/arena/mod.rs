use pyo3::{exceptions::PyValueError, prelude::*};

mod core;
use core::{Arena as RustArena, ArenaError};

#[pyclass]
pub struct Arena {
    inner: RustArena,
}

#[pymethods]
impl Arena {
    #[new]
    fn new(command1: Vec<String>, command2: Vec<String>) -> Self {
        Arena {
            inner: RustArena::new(command1, command2),
        }
    }

    fn play_n(&mut self, n: usize) -> PyResult<()> {
        match self.inner.play_n(n) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ArenaError::EngineStartError => Err(PyValueError::new_err("Engine start error")),
                ArenaError::GameNumberInvalid => Err(PyValueError::new_err("Game count must be even")),
                ArenaError::GameError(s) => Err(PyValueError::new_err(format!("Game error: {:?}", s))),
                _ => Err(PyValueError::new_err(format!("{:?}", e))),
            },
        }
    }

    fn get_stats(&self) -> (usize, usize, usize) {
        self.inner.get_stats()
    }

    fn get_pieces(&self) -> (usize, usize) {
        self.inner.get_pieces()
    }
}
