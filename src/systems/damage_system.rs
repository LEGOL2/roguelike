use specs::prelude::*;
use super::{CombatStats, SufferDamage, console, Player};

pub struct DamageSystem {}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entites = ecs.entities();
        for (entity, stats) in (&entites, &combat_stats).join() {
            if stats.hp < 1 { 
                let player = players.get(entity);
                match player {
                    Some(_) => console::log("You are dead!"),
                    None => dead.push(entity)
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Failed to delete dead entity.");
    }
}
