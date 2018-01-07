//! Detailed game errors.

/// Detailed game errors.
#[derive(Debug, Fail, PartialEq, Eq)]
pub enum GameError {
    /// The given unit does not have a type appropriate for the given action.
    #[fail(display = "The given unit does not have a type appropriate for the given action.")]
    InappropriateUnitType,

    /// The engine encountered a problem. Report this to the devs.
    #[fail(display = "The engine encountered a problem. Report this to the devs.")]
    InternalEngineError,

    /// The action you attempted to perform is not allowed. Did you check `can_action()` before performing `action()`?
    #[fail(display = "The action you attempted to perform is not allowed. Did you check can_action() before performing action()?")]
    InvalidAction,

    /// The map-related object is invalid.
    #[fail(display = "The map-related object is invalid.")]
    InvalidMapObject,

    /// The location is off the map or otherwise outside your vision range.
    #[fail(display = "The location is off the map or otherwise outside your vision range.")]
    InvalidLocation,

    /// You are not allowed to control units on the other team.
    #[fail(display = "You are not allowed to control units on the other team.")]
    TeamNotAllowed,

    /// You cannot read outside of the bounds of the communication array.
    #[fail(display = "You cannot read outside of the bounds of the communication array.")]
    ArrayOutOfBounds,

    /// You cannot build structures on Mars.
    #[fail(display = "You cannot build structures on Mars.")]
    CannotBuildOnMars,

    /// The factory is already producing a unit.
    #[fail(display = "The factory is already producing a unit.")]
    FactoryBusy,

    /// The structure's garrison is empty.
    #[fail(display = "The structure's garrison is empty.")]
    GarrisonEmpty,

    /// Your team does not have enough Karbonite to perform the requested action.
    #[fail(display = "Your team does not have enough Karbonite to perform the requested action.")]
    InsufficientKarbonite,

    /// The level of research may not exist, or has not been unlocked by your team.
    #[fail(display = "The level of research may not exist, or has not been unlocked by your team.")]
    InvalidResearchLevel,

    /// The Karbonite deposit is empty and cannot be harvested further.
    #[fail(display = "The Karbonite deposit is empty and cannot be harvested further.")]
    KarboniteDepositEmpty,

    /// The location corresponding to the requested action is not empty.
    #[fail(display = "The location corresponding to the requested action is not empty.")]
    LocationNotEmpty,

    /// The structure's garrison is full.
    #[fail(display = "The structure's garrison is full.")]
    NotEnoughSpace,

    /// The specified unit does not exist, at least within your vision range.
    #[fail(display = "The specified unit does not exist, at least within your vision range.")]
    NoSuchUnit,

    /// The unit is too far away to perform an action.
    #[fail(display = "The unit is too far away to perform an action.")]
    OutOfRange,

    /// The unit's heat is not low enough to perform the requested action.
    #[fail(display = "The unit's heat is not low enough to perform the requested action.")]
    Overheated,

    /// The rocket has already been used.
    #[fail(display = "The rocket has already been used.")]
    RocketUsed,

    /// Rockets cannot be flown to other locations on the same planet.
    #[fail(display = "Rockets cannot be flown to other locations on the same planet.")]
    SamePlanet,

    /// The structure has already been completed, and cannot be built further.
    #[fail(display = "The structure has already been completed, and cannot be built further.")]
    StructureAlreadyBuilt,

    /// The structure has not yet been completed, and cannot perform actions yet.
    #[fail(display = "The structure has not yet been completed, and cannot perform actions yet.")]
    StructureNotYetBuilt,

    /// The unit is not on the map.
    #[fail(display = "The unit is not on the map")]
    UnitNotOnMap,

    /// The argument to this function does not conform to the specs.
    #[fail(display = "The argument to this function does not conform to the specs.")]
    IllegalArgument,
}

/// Asserts that $left is an Err whose unwrapped value is the game error
/// $right. This macro is helpful since all of our Errors wrap GameErrors,
/// and we can use this to ensure it's the correct type of GameError.
#[cfg(test)]
macro_rules! assert_err {
    ($left:expr, $right:expr) => ({
        assert_eq!(
            $left.unwrap_err().downcast::<GameError>().expect("wrong error type"),
            $right
        )
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        assert_eq!(
            $left.unwrap_err().downcast::<GameError>().expect("wrong error type"),
            $right,
            format_args!($($arg)+)
        )
    });
}

/// Asserts that $left is less than or equal to $right. More informative than
/// assert!(left <= right), since it'll output the $left and $right values
/// when panicking.
#[cfg(test)]
macro_rules! assert_lte {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val <= *right_val) {
                    panic!(r#"assertion failed: `(left <= right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val <= *right_val) {
                    panic!(r#"assertion failed: `(left <= right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, left_val, right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
}

/// Asserts that $left is greater than $right. More informative than
/// assert!(left > right), since it'll output the $left and $right values
/// when panicking.
#[cfg(test)]
macro_rules! assert_gt {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val > *right_val) {
                    panic!(r#"assertion failed: `(left > right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val > *right_val) {
                    panic!(r#"assertion failed: `(left > right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, left_val, right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
}
