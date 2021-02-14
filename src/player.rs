use bracket_lib::prelude::{VirtualKeyCode, BTerm};
use specs::prelude::*;
use super::{Position, Player, TileType, xy_idx, State};
use std::cmp::{min, max};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (pos, _) in (&mut positions, &players).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

pub fn player_input(game_state: &mut State, ctx: &mut BTerm) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 => try_move_player(-1, 0, &mut game_state.ecs),
            
            VirtualKeyCode::Numpad7 =>try_move_player(-1, -1, &mut game_state.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => try_move_player(0, -1, &mut game_state.ecs),
            
            VirtualKeyCode::Numpad9 =>try_move_player(1, -1, &mut game_state.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 => try_move_player(1, 0, &mut game_state.ecs),
            
            VirtualKeyCode::Numpad3 =>try_move_player(1, 1, &mut game_state.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => try_move_player(0, 1, &mut game_state.ecs),

            VirtualKeyCode::Numpad1 =>try_move_player(-1, 1, &mut game_state.ecs),

            _ => {}
        },
    }
}
