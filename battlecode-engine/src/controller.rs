//! The outermost layer of the engine stack. Responsible for exposing
//! the API that the player will use, and for generating messages to
//! send to other parts of the Battlecode infrastructure.

use config::Config;
use location::*;
use schema::*;
use unit::*;
use world::GameWorld;

use failure::Error;

struct GameController {
    world: GameWorld,
    config: Config,
    turn: TurnMessage,
}

impl GameController {

    // ************************************************************************
    // ************************************************************************
    // ************************************************************************
    // **************************** PLAYER API ********************************
    // ************************************************************************
    // ************************************************************************
    // ************************************************************************

    /// Initializes the game world and creates a new controller
    /// for a player to interact with it.
    pub fn new() -> GameController {
        GameController {
            // TODO: load an actual map.
            world: GameWorld::test_world(),
            config: Config::player_config(),
            turn: TurnMessage { changes: vec![] }
        }
    }

    /// Starts the current turn, by updating the player's GameWorld with changes
    /// made since the last time the player had a turn.
    pub fn start_turn(&mut self, turn: StartTurnMessage) -> Result<(), Error> {
        self.world = turn.world;
        Ok(())
    }

    /// Ends the current turn. Returns the list of changes made in this turn.
    pub fn end_turn(&mut self) -> Result<TurnMessage, Error> {
        self.world.end_turn()?;
        // v Does this deep copy? v
        let turn = self.turn.clone();
        self.turn = TurnMessage { changes: vec![] };
        Ok(turn)
    }

    // *****************************************************************************
    // **************************** TESTING METHODS ********************************
    // *****************************************************************************

    /// Tests whether the given robot can move in the specified direction.
    /// Returns an error if the id does not correspond to a known robot, etc...
    pub fn can_move_robot(&self, robot_id: UnitID, direction: Direction) -> Result<bool, Error> {
        Ok(self.world.can_move(robot_id, direction)?)
    }

    // *********************************************************************
    // **************************** ACTIONS ********************************
    // *********************************************************************

    /// Commands the given robot to move one square in the specified
    /// direction. Returns an error if the move is unsuccessful, etc...
    pub fn move_robot(&mut self, robot_id: UnitID, direction: Direction) -> Result<(), Error> {
        let delta = Delta::Move { robot_id, direction };
        if self.config.generate_turn_messages {
            self.turn.changes.push(delta.clone());
        }
        Ok(self.world.apply(&delta)?)
    }

    // TODO: wrappers for all of the other functions.

    // ************************************************************************
    // ************************************************************************
    // ************************************************************************
    // **************************** RUNNER API ********************************
    // ************************************************************************
    // ************************************************************************
    // ************************************************************************

    /// Initializes the game world and creates a new controller
    /// for the runner to interact with it.
    pub fn new_runner() -> GameController {
        GameController {
            // TODO: load an actual map.
            world: GameWorld::test_world(),
            config: Config::runner_config(),
            turn: TurnMessage { changes: vec![] }
        }
    }

    /// Given a TurnMessage from a player, apply those changes.
    pub fn apply_turn(&mut self, turn: TurnMessage) -> Result<(StartTurnMessage, ViewerMessage), Error> {
        self.world.apply_turn(&turn)?;
        // Serialize the filtered game state to send to the player
        // TODO: filter the world somehow
        let start_turn_message = StartTurnMessage { world: self.world.clone() };
        // Serialize the game state to send to the viewer
        let viewer_message = ViewerMessage { world: self.world.clone() };
        Ok((start_turn_message, viewer_message))
    }
}