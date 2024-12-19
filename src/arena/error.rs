#[derive(Debug)]
pub enum PlayerError {
    IoError,
    ParseError,
    TimeoutError,
    BoardError,
}

#[derive(Debug)]
pub enum GameError {
    BlackInvalidMove,
    WhiteInvalidMove,
    BlackTimeout,
    WhiteTimeout,
    BlackCrash,
    WhiteCrash,
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
    IoError,
}

impl From<std::io::Error> for ClientManagerError {
    fn from(_: std::io::Error) -> Self {
        ClientManagerError::IoError
    }
    
}

#[derive(Debug)]
pub enum NetworkArenaServerError {
    IoError(std::io::Error),
    ClientManagerError(ClientManagerError),
    ClientNotReady,
}

impl From<std::io::Error> for NetworkArenaServerError {
    fn from(_: std::io::Error) -> Self {
        NetworkArenaServerError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "IO error"))
    }
}

impl From<ClientManagerError> for NetworkArenaServerError {
    fn from(e: ClientManagerError) -> Self {
        NetworkArenaServerError::ClientManagerError(e)
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
