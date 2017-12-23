#[derive(Debug, Fail, PartialEq, Eq)]
pub enum GameError {
    /// The engine encountered a problem. Report this to the devs.
    #[fail(display = "The engine encountered a problem. Report this to the devs.")]
    InternalEngineError,

    /// The action you attempted to perform is not allowed.
    #[fail(display = "The action you attempted to perform is not allowed.")]
    InvalidAction,

    /// The specified unit does not exist.
    #[fail(display = "The specified unit does not exist.")]
    NoSuchUnit,
}