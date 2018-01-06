extern crate battlecode_engine as bc;
extern crate failure;
extern crate rand;
extern crate serde_json;

use bc::controller::*;
use bc::schema::*;
use bc::location::*;
use bc::unit::*;
use bc::world::*;
use failure::Error;
use std::fs::File;
use std::io::Write;
use rand::Rng;

mod jamesplayer;
mod badplayer;

// Simple runner for players written in rust while we work on fusing other components

fn run() -> Result<(), Error> {
    let mut manager = GameController::new_manager();

    manager.world.create_unit(Team::Red, MapLocation::new(Planet::Earth, 0, 0), UnitType::Worker)?;
    manager.world.create_unit(Team::Blue, MapLocation::new(Planet::Earth, 19, 19), UnitType::Worker)?;

    let mut p1ec = GameController::new();
    let mut p1e = jamesplayer::Player::new();
    let mut p1mc = GameController::new();
    let mut p1m = jamesplayer::Player::new();

    let mut p2ec = GameController::new();
    let mut p2e = badplayer::Player::new();
    let mut p2mc = GameController::new();
    let mut p2m = badplayer::Player::new();

    let mut rng = rand::thread_rng();
    let mut outfile = File::create(format!("game{}.json", rng.gen::<u32>()))?;

    let mut message = manager.first_turn()?;

    for _ in 0..1000 {
        p1ec.start_turn(message)?;
        p1e.make_turn(&mut p1ec)?;
        let backmessage = p1ec.end_turn()?;
        let (message_, vmesg) = manager.apply_turn(backmessage)?;
        serde_json::to_writer(&mut outfile, &vmesg)?;
        outfile.write_all(b"\n")?;
        drop(vmesg);

        p2ec.start_turn(message_)?;
        p2e.make_turn(&mut p2ec)?;
        let backmessage = p2ec.end_turn()?;
        let (message_, vmesg) = manager.apply_turn(backmessage)?;
        serde_json::to_writer(&mut outfile, &vmesg)?;
        outfile.write_all(b"\n")?;
        drop(vmesg);
 
        p1mc.start_turn(message_)?;
        p1m.make_turn(&mut p1mc)?;
        let backmessage = p1mc.end_turn()?;
        let (message_, vmesg) = manager.apply_turn(backmessage)?;
        serde_json::to_writer(&mut outfile, &vmesg)?;
        outfile.write_all(b"\n")?;
        drop(vmesg);
 
        p2mc.start_turn(message_)?;
        p2m.make_turn(&mut p2mc)?;
        let backmessage = p2mc.end_turn()?;
        let (message_, vmesg) = manager.apply_turn(backmessage)?;
        serde_json::to_writer(&mut outfile, &vmesg)?;
        outfile.write_all(b"\n")?;
        drop(vmesg);
        message = message_;
    }

    Ok(())
}

fn main() {
    run().unwrap()
}

