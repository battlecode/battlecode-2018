//! Configuration for the engine.

pub struct Config {
    /// Whether to generate messages to be sent to the viewer.
    pub generate_viewer_messages: bool,

    /// Whether to generate turn messages.
    pub generate_turn_messages: bool,
}

impl Config {
    pub fn player_config() -> Config {
        Config {
            generate_viewer_messages: false,
            generate_turn_messages: true,
        }
    }

    pub fn runner_config() -> Config {
        Config {
            generate_viewer_messages: true,
            generate_turn_messages: true,
        }
    }
}