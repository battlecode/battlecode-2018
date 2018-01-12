//! Detailed game errors.
use super::research::Level;
use super::unit::UnitType;

/// Detailed game errors.
#[derive(Debug, Fail, PartialEq, Eq)]
pub enum GameError {
    /// You cannot read outside of the bounds of the communication array.
    #[fail(display = "You cannot read outside of the bounds of the communication array.")]
    ArrayOutOfBounds,

    /// You cannot build structures on Mars.
    #[fail(display = "You cannot build structures on Mars.")]
    CannotBuildOnMars,

    /// The locations are on different planets.
    #[fail(display = "The locations are on different planets.")]
    DifferentPlanet,

    /// The factory is already producing a unit.
    #[fail(display = "The factory is already producing a unit.")]
    FactoryBusy,

    /// The structure's garrison is empty.
    #[fail(display = "The structure's garrison is empty.")]
    GarrisonEmpty,

    /// The structure's garrison is full.
    #[fail(display = "The structure's garrison is full.")]
    GarrisonFull,

    /// The given unit does not have a type appropriate for the given action.
    #[fail(display = "The given unit does not have a type appropriate for the given action.")]
    InappropriateUnitType,

    /// The map-related object is invalid.
    #[fail(display = "The map-related object is invalid.")]
    InvalidMapObject,

    /// Your team does not have enough Karbonite to perform the requested action.
    #[fail(display = "Your team does not have enough Karbonite to perform the requested action.")]
    InsufficientKarbonite,

    /// The Karbonite deposit is empty and cannot be harvested further.
    #[fail(display = "The Karbonite deposit is empty and cannot be harvested further.")]
    KarboniteDepositEmpty,

    /// The location corresponding to the requested action is not empty.
    #[fail(display = "The location corresponding to the requested action is not empty.")]
    LocationNotEmpty,

    /// The location is outside your vision range.
    #[fail(display = "The location is outside your vision range.")]
    LocationNotVisible,

    /// The location is off the map of the current planet.
    #[fail(display = "The location is off the map of the current planet.")]
    LocationOffMap,

    /// The specified unit does not exist, at least within your vision range.
    #[fail(display = "The specified unit does not exist, at least within your vision range.")]
    NoSuchUnit,

    /// No object returned, check whether it exists first.
    #[fail(display = "No object returned, check whether it exists first.")]
    NullValue,

    /// The unit is too far away to perform an action.
    #[fail(display = "The unit is too far away to perform an action.")]
    OutOfRange,

    /// The unit's heat is not low enough to perform the requested action.
    #[fail(display = "The unit's heat is not low enough to perform the requested action.")]
    Overheated,

    /// The level of research does not exist for this branch.
    #[fail(display = "The level of research does not exist for this branch.")]
    ResearchLevelInvalid,

    /// The level of research has not been unlocked by your team.
    #[fail(display = "The level of research has not been unlocked by your team for the unit {:?}.", unit_type)]
    ResearchNotUnlocked { unit_type: UnitType },

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

    /// You are not allowed to control units on the other team.
    #[fail(display = "You are not allowed to control units on the other team.")]
    TeamNotAllowed,

    /// The unit is in a structure's garrison or flying through space.
    #[fail(display = "The unit is in a structure's garrison or flying through space.")]
    UnitNotOnMap,

    /// The unit is not in a structure's garrison.
    #[fail(display = "The unit is not in a structure's garrison.")]
    UnitNotInGarrison,
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

/// Asserts that $left is greater than or equal to $right. More informative
/// than assert!(left <= right), since it'll output the $left and $right values
/// when panicking.
#[cfg(test)]
macro_rules! assert_gte {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val >= *right_val) {
                    panic!(r#"assertion failed: `(left >= right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val >= *right_val) {
                    panic!(r#"assertion failed: `(left >= right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, left_val, right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
}

/// Asserts that $left is less than $right. More informative than
/// assert!(left <= right), since it'll output the $left and $right values
/// when panicking.
#[cfg(test)]
macro_rules! assert_lt {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val < *right_val) {
                    panic!(r#"assertion failed: `(left < right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val < *right_val) {
                    panic!(r#"assertion failed: `(left < right)`
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
