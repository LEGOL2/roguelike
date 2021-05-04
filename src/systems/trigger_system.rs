use super::{
    gamelog::GameLog, EntityMoved, EntryTrigger, Hidden, InflictsDamage, Map, Name,
    ParticleBuilder, Position, SingleActivation, SufferDamage,
};
use bracket_lib::prelude::*;
use specs::prelude::*;

pub struct TriggerSystem;

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            mut hidden,
            names,
            entities,
            mut log,
            inflicts_damage,
            mut particle_builder,
            mut inflict_damage,
            single_activation,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    if let Some(_) = maybe_trigger {
                        let name = names.get(*entity_id);
                        if let Some(name) = name {
                            log.entries.push(format!("{} triggers!", &name.name));
                        }

                        hidden.remove(*entity_id);

                        let damage = inflicts_damage.get(*entity_id);
                        if let Some(damage) = damage {
                            particle_builder.request(
                                pos.x,
                                pos.y,
                                RGB::named(ORANGE),
                                RGB::named(BLACK),
                                to_cp437('‼'),
                                200.0,
                            );
                            SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage);
                        }

                        let sa = single_activation.get(*entity_id);
                        if let Some(_) = sa {
                            remove_entities.push(*entity_id);
                        }
                    }
                }
            }
        }
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Failed to delete a trap.");
        }

        entity_moved.clear();
    }
}
