use crate::{CombatStats, RunState, WantsToMelee};

use super::{Map, Player, Position, State, Viewshed};
use bracket_lib::prelude::{BTerm, Point, VirtualKeyCode};
use specs::prelude::*;
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

            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}
