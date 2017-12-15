pub enum GameError {
    /// The engine encountered a problem. Report this to the devs.
    InternalEngineError,

    /// The specified entity does not exist.
    NoSuchEntity,
}