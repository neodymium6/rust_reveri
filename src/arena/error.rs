#[derive(Debug)]
pub enum PlayerError {
    Io,
    Parse,
    Timeout,
    Board,
}

#[derive(Debug)]
pub enum GameError {
    BlackInvalidMove,
    WhiteInvalidMove,
    BlackTimeout,
    WhiteTimeout,
    BlackCrash,
    WhiteCrash,
    GameNotOverYet,
    UnexpectedError,
}

#[derive(Debug)]
pub enum ArenaError {
    EngineStartError,
    EngineEndError,
    GameNumberInvalid,
    ThreadJoinError,
    GameError(GameError),
}

#[derive(Debug)]
pub enum ClientManagerError {
    NoMoreClients,
    ClientNotExists,
    IoError(std::io::Error),
    UnexpectedResponse,
}

impl From<std::io::Error> for ClientManagerError {
    fn from(e: std::io::Error) -> Self {
        ClientManagerError::IoError(e)
    }
}

#[derive(Debug)]
pub enum NetworkArenaServerError {
    IoError(std::io::Error),
    ClientManagerError(ClientManagerError),
    ClientNotReady,
    GameNumberInvalid,
    ArenaError(ArenaError),
}

impl From<std::io::Error> for NetworkArenaServerError {
    fn from(e: std::io::Error) -> Self {
        NetworkArenaServerError::IoError(e)
    }
}

impl From<ClientManagerError> for NetworkArenaServerError {
    fn from(e: ClientManagerError) -> Self {
        NetworkArenaServerError::ClientManagerError(e)
    }
}

impl From<ArenaError> for NetworkArenaServerError {
    fn from(e: ArenaError) -> Self {
        NetworkArenaServerError::ArenaError(e)
    }
}

#[derive(Debug)]
pub enum NetworkArenaClientError {
    IoError(std::io::Error),
    ConnectionBroken,
    UnexpectedServerResponse,
}

impl From<std::io::Error> for NetworkArenaClientError {
    fn from(e: std::io::Error) -> Self {
        NetworkArenaClientError::IoError(e)
    }
}
