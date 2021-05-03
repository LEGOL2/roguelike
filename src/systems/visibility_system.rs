use specs::prelude::*;

use super::{Map, Player, Position, Viewshed, Hidden, Name, GameLog};
use bracket_lib::prelude::{field_of_view, Point, RandomNumberGenerator};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, mut hidden, mut rng, mut log, names) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                let p = player.get(ent);
                if let Some(_p) = p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        for e in map.tile_content[idx].iter() {
                            let maybe_hidden = hidden.get(*e);
                            if let Some(_) = maybe_hidden {
                                if rng.roll_dice(1, 30) == 1 {
                                    let name = names.get(*e);
                                    if let Some(name) = name {
                                        log.entries.push(format!("You've spotted a {}.", &name.name));
                                    }
                                    hidden.remove(*e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
