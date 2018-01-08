//! The "schema" for battlecode: all messages that can be sent to and from the engine.
//! Serialized to JSON using Serde. This results in message parsers that are as fast
//! as handwritten message parsing.
//!
//! The mesages in a typical game between the manager and a player:
//! Manager --StartGameMessage--> Red Earth
//! Manager <----TurnMessage----- Red Earth
//! Manager --StartTurnMessage--> Red Earth
//! Manager <----TurnMessage----- Red Earth

use super::id_generator::*;
use super::location::*;
use super::research::*;
use super::rockets::*;
use super::unit::*;
use super::world::*;

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
    /// Commands the given factory to produce a robot.
    ProduceRobot { factory_id: UnitID, robot_type: UnitType },
    /// Queues the next level of the given research branch.
    QueueResearch { branch: UnitType },
    /// Commands the given worker to repair the specified strucutre.
    Repair { worker_id: UnitID, structure_id: UnitID },
    /// Commands the given worker to replicate in the given direction.
    Replicate { worker_id: UnitID, direction: Direction },
    /// Resets the current research queue, for the specified team.
    ResetResearchQueue,
    /// Commands the given structure to unload a unit in the given direction.
    Unload { structure_id: UnitID, direction: Direction },
    /// Writes the value at the index of this player's team array.
    WriteTeamArray { index: usize, value: i32 },
    /// Nothing happens.
    Nothing,
}

/// The first message sent to each player by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartGameMessage {
    /// The initial filtered world.
    pub world: GameWorld,
}

/// A message sent to the viewer which contains all the information needed
/// to initialize or reinitialize the viewer's state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewerKeyframe {
    /// A full copy of the initial game state.
    pub world: GameWorld,
}

/// A single game turn sent to the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnMessage {
    /// The changes to the game world.
    pub changes: Vec<Delta>
}

/// A list of updates since the player's last turn sent to the player.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartTurnMessage {
    pub round: Rounds,

    // PlanetInfo
    pub visible_locs: Vec<Vec<bool>>,
    pub units_changed: Vec<Unit>,
    pub units_vanished: Vec<UnitID>,
    pub unit_infos_changed: Vec<UnitInfo>,
    pub unit_infos_vanished: Vec<UnitID>,
    pub karbonite_changed: Vec<(MapLocation, u32)>,

    // TeamInfo
    pub id_generator: IDGenerator,
    pub units_in_space_changed: Vec<Unit>,
    pub units_in_space_vanished: Vec<UnitID>,
    pub other_array_changed: Vec<(usize, i32)>,
    pub rocket_landings: RocketLandingInfo,
    pub research: ResearchInfo,
    pub karbonite: u32,
}

/// The truncated unit info needed by the viewer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewerUnitInfo {
    pub id: UnitID,
    pub unit_type: UnitType,
    pub health: u32,
    pub location: Location,
}

/// Additional information that the viewer may need.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewerDelta {
    AsteroidStrike { location: MapLocation },
    RocketLanding { rocket_id: UnitID, location: MapLocation },
    RangerSnipe { ranger_id: UnitID, target_location: MapLocation },
}

/// A description of the current game state, for the viewer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewerMessage {
    /// The current status of the GameWorld.
    pub changes: Vec<Delta>,
    pub units: Vec<ViewerUnitInfo>,
    pub additional_changes: Vec<ViewerDelta>,
}

/// An error message in response to some error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// The error string.
    pub error: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginMessage {
    pub client_id: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReceivedMessage<T> {
    pub logged_in: bool,
    pub client_id: String,
    pub error: Option<String>,
    pub message: Option<T>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SentMessage {
    pub client_id: String,
    pub turn_message: TurnMessage
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
