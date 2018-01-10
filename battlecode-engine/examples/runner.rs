extern crate battlecode_engine as bc;
extern crate failure;
extern crate rand;

use std::env;
use std::collections::HashMap;

use bc::controller::*;
use bc::world::*;
use bc::unit::*;
use bc::map::*;
use bc::location::*;

use Location::*;
use Team::*;
use Direction::*;
use UnitType::*;

use failure::Error;

use rand::{Rng, SeedableRng, ChaChaRng};

fn examplefuncsplayer(gc: &mut GameController) -> Result<(), Error> {
    let mut rng = ChaChaRng::from_seed(&[2284860895, 1790736221, 1190208258, 3279695007, 2888369390, 2233370644, 3161697024, 2177838068]);
    let alld = Direction::all();

    'unit: for unit in gc.my_units() {
        if let OnMap(loc) = unit.location() {
            let nearby = gc.sense_nearby_units(loc, 2);
            for other in nearby {
                if unit.unit_type() == UnitType::Worker && gc.can_build(unit.id(), other.id()) {
                    gc.build(unit.id(), other.id())?;
                    println!("building {} {}", unit.id(), other.id());
                    continue 'unit;
                }
            }
        }
        for _ in 0..4 {
            let dir = *rng.choose(&alld[..]).unwrap();
            if rng.gen::<u8>() < 128 && gc.karbonite() > UnitType::Factory.blueprint_cost()? &&
                gc.can_blueprint(unit.id(), UnitType::Factory, dir) {
                println!("blueprinting {:?} {:?}", unit.location(), dir);
                gc.blueprint(unit.id(), UnitType::Factory, dir)?;
                break;
            } else if gc.is_move_ready(unit.id()) && gc.can_move(unit.id(), dir) {
                println!("moving {:?} {:?}", unit.location(), dir);
                gc.move_robot(unit.id(), dir)?;
                break;
            }
        }
    }
    Ok(())
}

fn nothingbot(gc: &mut GameController) -> Result<(), Error> {
    Ok(())
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 4 {
        println!("usage: runner P1 P2 DELAYMS");
        return;
    }
    let p1 = match &args[1][..] {
        "examplefuncsplayer" => examplefuncsplayer,
        "nothingbot" => nothingbot,
        _ => panic!("unknown bot")
    };
    let p2 = match &args[2][..] {
        "examplefuncsplayer" => examplefuncsplayer,
        "nothingbot" => nothingbot,
        _ => panic!("unknown bot")
    };
    let delay = args[3].parse::<u32>().unwrap();
    run_game_ansi(p1, p2, 1000, delay);
}