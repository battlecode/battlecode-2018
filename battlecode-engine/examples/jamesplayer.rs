use failure::Error;
use bc::location::Direction;
use bc::controller::GameController;

pub struct Player {
    anger: u32
}

impl Player {
    pub fn new() -> Player {
        Player { anger: 0 }
    }
    
    pub fn make_turn(&mut self, controller: &mut GameController) -> Result<(), Error> {
        if controller.round() % 10 == 0 {
            println!("jamesplayer team: {:?}, round: {:?}, planet: {:?}, anger: {:?}",
                controller.team(),
                controller.round(),
                controller.planet(),
                self.anger
            );
            self.anger += 1;
        }
        let team = controller.team();
        let first_id = controller.units().first().and_then(|u| {
            if u.team == team {
                Some(u.id)
            } else {
                None
            }
        });
        if let Some(id) = first_id {
            controller.move_robot(id, Direction::North);
        }
        Ok(())
    }
}