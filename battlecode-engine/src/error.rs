pub enum GameError {
    /// The engine encountered a problem. Report this to the devs.
    InternalEngineError,

    /// The action you attempted to perform is not allowed.
    InvalidAction,

    /// The specified entity does not exist.
    NoSuchEntity,
}