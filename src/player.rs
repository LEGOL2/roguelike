use super::*;
use bracket_lib::prelude::{BTerm, Point, VirtualKeyCode};
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (pos, _, viewshed, entity) in (&mut positions, &players, &mut viewsheds, &entities).join() {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Failed to add a target.");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input(game_state: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            // Movement
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 => {
                try_move_player(-1, 0, &mut game_state.ecs)
            }

            VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut game_state.ecs),

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
                try_move_player(0, -1, &mut game_state.ecs)
            }

            VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut game_state.ecs),

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 => {
                try_move_player(1, 0, &mut game_state.ecs)
            }

            VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut game_state.ecs),

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
                try_move_player(0, 1, &mut game_state.ecs)
            }

            VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut game_state.ecs),

            // Pickup item
            VirtualKeyCode::G => get_item(&mut game_state.ecs),

            // Open inventory
            VirtualKeyCode::B => return RunState::ShowInventory,

            // Drop item
            VirtualKeyCode::D => return RunState::ShowDropItem,

            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<gamelog::GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item: item,
                    },
                )
                .expect("Unable to insert WantToPickupItem component");
        }
        None => gamelog
            .entries
            .push("There is nothing to pick up.".to_string()),
    }
}
