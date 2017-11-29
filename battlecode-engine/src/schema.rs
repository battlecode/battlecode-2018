//! The "schema" for battlecode: all messages that can be sent to and from the engine.
//! Serialized to JSON using Serde. This results in message parsers that are as fast
//! as handwritten message parsing.

use std::fmt;

/// A single, atomic "change" in the game world.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Delta {
    /// Nothing happens.
    Nothing,
    
    /// Dummy implementation: increment the game count by some amount
    Increment { amount: u32 }
}

/// A single game turn.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TurnMessage {
    /// The changes to the game world.
    changes: Vec<Delta>
}

/// An error message in response to some error.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ErrorMessage {
    /// The error string.
    error: String
}

#[cfg(test)]
mod tests {
    use failure::Error;
    use super::*;
    use serde_json::{from_str, to_str};

    #[test]
    fn roundtrip() {
        let turn = TurnMessage {
            changes: vec![Delta::Nothing]
        };
        assert!(from_str(to_str(&turn)) == turn);

        let error = ErrorMessage {
            error: "bees are attacking".into()
        };
        assert!(from_str(to_str(&error)) == error);
    }
}
