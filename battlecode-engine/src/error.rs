#[derive(Debug, Fail, PartialEq, Eq)]
pub enum GameError {
    /// The engine encountered a problem. Report this to the devs.
    #[fail(display = "The engine encountered a problem. Report this to the devs.")]
    InternalEngineError,

    /// The action you attempted to perform is not allowed.
    #[fail(display = "The action you attempted to perform is not allowed.")]
    InvalidAction,

    /// The specified planet does not exist. This is probably the devs' fault.
    #[fail(display = "The specified planet does not exist. This is probably the devs' fault.")]
    NoSuchPlanet,

    /// The specified unit does not exist.
    #[fail(display = "The specified unit does not exist.")]
    NoSuchUnit,
}