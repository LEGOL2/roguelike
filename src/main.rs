use bracket_lib::prelude::*;
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod systems;
pub use systems::{visibility_system::*};

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;

    let mut game_state = State { ecs: World::new() };
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();
    game_state.ecs.insert(map);

    game_state
        .ecs
        .create_entity()
        .with(Position { x: player_pos.x, y: player_pos.y })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .build();

    main_loop(context, game_state)
}

pub struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

