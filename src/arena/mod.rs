use pyo3::{exceptions::PyValueError, prelude::*};

use rust_reversi_core::arena::LocalArena as RustLocalArena;
use rust_reversi_core::arena::{
    ArenaError, ClientManagerError, NetworkArenaClientError, NetworkArenaServerError,
};
use rust_reversi_core::arena::{
    NetworkArenaClient as RustNetworkArenaClient, NetworkArenaServer as RustNetworkArenaServer,
};

#[pyclass]
pub struct Arena {
    inner: RustLocalArena,
}

#[pymethods]
impl Arena {
    #[new]
    #[pyo3(signature = (command1, command2, show_progress=None))]
    fn new(command1: Vec<String>, command2: Vec<String>, show_progress: Option<bool>) -> Self {
        Arena {
            inner: RustLocalArena::new(command1, command2, show_progress.unwrap_or(true)),
        }
    }

    fn play_n(&mut self, n: usize) -> PyResult<()> {
        match self.inner.play_n(n) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ArenaError::EngineStartError => Err(PyValueError::new_err("Engine start error")),
                ArenaError::GameNumberInvalid => {
                    Err(PyValueError::new_err("Game count must be even"))
                }
                ArenaError::EngineEndError => Err(PyValueError::new_err("Engine end error")),
                ArenaError::ThreadJoinError => Err(PyValueError::new_err("Thread join error")),
                ArenaError::GameError(s) => {
                    Err(PyValueError::new_err(format!("Game error: {:?}", s)))
                }
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

#[pyclass]
pub struct NetworkArenaServer {
    inner: RustNetworkArenaServer,
}

#[pymethods]
impl NetworkArenaServer {
    #[new]
    #[pyo3(signature = (game_per_iter, show_progress=None))]
    fn new(game_per_iter: usize, show_progress: Option<bool>) -> PyResult<Self> {
        match RustNetworkArenaServer::new(game_per_iter, show_progress.unwrap_or(true)) {
            Ok(inner) => Ok(NetworkArenaServer { inner }),
            Err(e) => match e {
                NetworkArenaServerError::IoError(e) => Err(PyValueError::new_err(format!(
                    "NetworkArenaServer error: {:?}",
                    e
                ))),
                NetworkArenaServerError::ClientManagerError(e) => Err(PyValueError::new_err(
                    format!("NetworkArenaServer error: {:?}", e),
                )),
                NetworkArenaServerError::ClientNotReady => {
                    Err(PyValueError::new_err("Client not ready"))
                }
                NetworkArenaServerError::GameNumberInvalid => {
                    Err(PyValueError::new_err("Game count must be even"))
                }
                NetworkArenaServerError::ArenaError(e) => Err(PyValueError::new_err(format!(
                    "NetworkArenaServer error: {:?}",
                    e
                ))),
            },
        }
    }

    fn start(&mut self, py: Python<'_>, addr: String, port: u16) -> PyResult<()> {
        py.allow_threads(|| match self.inner.start(addr, port) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                NetworkArenaServerError::IoError(e) => Err(PyValueError::new_err(format!(
                    "NetworkArenaServer error: {:?}",
                    e
                ))),
                NetworkArenaServerError::ClientManagerError(e) => match e {
                    ClientManagerError::NoMoreClients => {
                        Err(PyValueError::new_err("No more clients"))
                    }
                    ClientManagerError::ClientNotExists => {
                        Err(PyValueError::new_err("Client not exists"))
                    }
                    ClientManagerError::IoError(e) => Err(PyValueError::new_err(format!(
                        "NetworkArenaServer error: {:?}",
                        e
                    ))),
                    ClientManagerError::UnexpectedResponse => {
                        Err(PyValueError::new_err("Unexpected response"))
                    }
                },
                NetworkArenaServerError::ClientNotReady => {
                    Err(PyValueError::new_err("Client not ready"))
                }
                NetworkArenaServerError::GameNumberInvalid => {
                    Err(PyValueError::new_err("Game count must be even"))
                }
                NetworkArenaServerError::ArenaError(e) => Err(PyValueError::new_err(format!(
                    "NetworkArenaServer error: {:?}",
                    e
                ))),
            },
        })
    }
}

#[pyclass]
pub struct NetworkArenaClient {
    inner: RustNetworkArenaClient,
}

#[pymethods]
impl NetworkArenaClient {
    #[new]
    fn new(command: Vec<String>) -> Self {
        NetworkArenaClient {
            inner: RustNetworkArenaClient::new(command),
        }
    }

    fn connect(&mut self, py: Python<'_>, addr: String, port: u16) -> PyResult<()> {
        py.allow_threads(|| match self.inner.connect(addr, port) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                NetworkArenaClientError::IoError(e) => Err(PyValueError::new_err(format!(
                    "NetworkArenaClient error: {:?}",
                    e
                ))),
                NetworkArenaClientError::ConnectionBroken => {
                    Err(PyValueError::new_err("Connection broken"))
                }
                NetworkArenaClientError::UnexpectedServerResponse => {
                    Err(PyValueError::new_err("Unexpected server response"))
                }
            },
        })
    }

    fn get_stats(&self) -> (usize, usize, usize) {
        self.inner.get_stats()
    }

    fn get_pieces(&self) -> (usize, usize) {
        self.inner.get_pieces()
    }
}
