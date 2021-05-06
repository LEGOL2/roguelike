use crate::{Map, Position};
use bracket_lib::prelude::RandomNumberGenerator;
use specs::prelude::World;

mod bsp_map;
pub mod common;
mod simple_map;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 2);
    match builder {
        1 => Box::new(bsp_map::BspMapBuilder::new(new_depth)),
        _ => Box::new(simple_map::SimpleMapBuilder::new(new_depth))
    }    
}
