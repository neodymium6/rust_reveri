use pyo3::{exceptions::PyValueError, prelude::*};

mod local;
mod network;
mod error;
mod core;
use error::ArenaError;
use local::LocalArena as RustLocalArena;

#[pyclass]
pub struct Arena {
    inner: RustLocalArena,
}

#[pymethods]
impl Arena {
    #[new]
    fn new(command1: Vec<String>, command2: Vec<String>) -> Self {
        Arena {
            inner: RustLocalArena::new(command1, command2),
        }
    }

    fn play_n(&mut self, n: usize) -> PyResult<()> {
        match self.inner.play_n(n) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ArenaError::EngineStartError => Err(PyValueError::new_err("Engine start error")),
                ArenaError::GameNumberInvalid => Err(PyValueError::new_err("Game count must be even")),
                ArenaError::EngineEndError => Err(PyValueError::new_err("Engine end error")),
                ArenaError::ThreadJoinError => Err(PyValueError::new_err("Thread join error")),
                ArenaError::GameError(s) => Err(PyValueError::new_err(format!("Game error: {:?}", s))),
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
