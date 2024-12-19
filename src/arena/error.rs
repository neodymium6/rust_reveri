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
