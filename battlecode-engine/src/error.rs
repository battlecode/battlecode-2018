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

    /// The level of research may not exist, or has not been unlocked by your team.
    #[fail(display = "The level of research may not exist, or has not been unlocked by your team.")]
    InvalidResearchLevel,

    /// The specified unit does not exist, at least within your vision range.
    #[fail(display = "The specified unit does not exist, at least within your vision range.")]
    NoSuchUnit,

    /// The argument to this function does not conform to the specs.
    #[fail(display = "The argument to this function does not conform to the specs.")]
    IllegalArgument,
}

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
