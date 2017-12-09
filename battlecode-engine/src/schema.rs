//! The "schema" for battlecode: all messages that can be sent to and from the engine.
//! Serialized to JSON using Serde. This results in message parsers that are as fast
//! as handwritten message parsing.

use std::fmt;
use super::location::*;
use super::entity::*;

/// A single, atomic "change" in the game world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Delta {
    /// Moves a robot with the given ID.
    Move { id: EntityID, location: MapLocation },
    /// Nothing happens.
    Nothing,
}

/// A single game turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnMessage {
    /// The changes to the game world.
    changes: Vec<Delta>
}

/// An error message in response to some error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// The error string.
    error: String
}

#[cfg(test)]
mod tests {
    use failure::Error;
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn turn_round_trip() {
        let turn = TurnMessage {
            changes: vec![Delta::Nothing]
        };
        let serialized = to_string(&turn).expect("failed to serialize");
        let deserialized: TurnMessage = from_str(&serialized).expect("failed to deserialize");
        assert_eq!(deserialized, turn);
    }

    #[test]
    fn error_round_trip() {
        let error = ErrorMessage {
            error: "bees are attacking".into()
        };
        let serialized = to_string(&error).expect("failed to serialize");
        let deserialized: ErrorMessage = from_str(&serialized).expect("failed to deserialize");
        assert_eq!(deserialized, error);
    }
}
