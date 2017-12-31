//! The "schema" for battlecode: all messages that can be sent to and from the engine.
//! Serialized to JSON using Serde. This results in message parsers that are as fast
//! as handwritten message parsing.

use super::location::*;
use super::unit::*;
use super::world::GameWorld;
use super::world::Team;

/// A single, atomic "change" in the game world.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Delta {
    /// Commands the given robot to attack a location.
    Attack { robot_id: UnitID, target_unit_id: UnitID },
    /// Commands the given ranger to begin sniping the given location.
    BeginSnipe { ranger_id: UnitID, location: MapLocation },
    /// Commands the given worker to blueprint a structure.
    Blueprint { worker_id: UnitID, structure_type: UnitType, direction: Direction },
    /// Commands the given mage to blink to the given location.
    Blink { mage_id: UnitID, location: MapLocation },
    /// Commands the given worker to build a blueprint.
    Build { worker_id: UnitID, blueprint_id: UnitID },
    /// Commands the given unit to disintegrate.
    Disintegrate { unit_id: UnitID },
    /// Commands the given worker to mine karbonite from an adjacent square.
    Harvest { worker_id: UnitID, direction: Direction },
    /// Commands the given healer to heal the given robot.
    Heal { healer_id: UnitID, target_robot_id: UnitID },
    /// Commands the given knight to throw a javelin at the given location.
    Javelin { knight_id: UnitID, target_unit_id: UnitID },
    /// Commands the given rocket to launch, ultimately landing in the specified location.
    LaunchRocket { rocket_id: UnitID, location: MapLocation },
    /// Commands the given structure to load the specified robot into its garrison.
    Load { structure_id: UnitID, robot_id: UnitID },
    /// Commands the given robot to move in the given direction.
    Move { robot_id: UnitID, direction: Direction },
    /// Commands the given healer to overcharge the specified robot.
    Overcharge { healer_id: UnitID, target_robot_id: UnitID },
    /// Queues the next level of the given research branch, for the specified team.
    QueueResearch { team: Team, branch: UnitType },
    /// Commands the given factory to enqueue production a robot.
    QueueRobotProduction { factory_id: UnitID, robot_type: UnitType },
    /// Commands the given worker to repair the specified strucutre.
    Repair { worker_id: UnitID, structure_id: UnitID },
    /// Commands the given worker to replicate in the given direction.
    Replicate { worker_id: UnitID, direction: Direction },
    /// Resets the current research queue, for the specified team.
    ResetResearchQueue { team: Team },
    /// Commands the given structure to unload a unit in the given direction.
    Unload { structure_id: UnitID, direction: Direction },
    /// Nothing happens.
    Nothing,
}

/// A single game turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnMessage {
    /// The changes to the game world.
    pub changes: Vec<Delta>
}

/// A list of updates since the player's last turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartTurnMessage {
    /// The current status of the GameWorld.
    pub world: GameWorld
}

/// A description of the current game state, for the viewer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewerMessage {
    /// The current status of the GameWorld.
    pub world: GameWorld
}

/// An error message in response to some error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// The error string.
    error: String
}

#[cfg(test)]
mod tests {
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
