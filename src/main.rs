use bracket_lib::prelude::*;
use gamelog::GameLog;
use gui::{MainMenuResult, MainMenuSelection};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod systems;
pub use systems::{
    damage_system::*, inventory_system::*, map_indexing_system::*, melee_combat_system::*,
    monster_ai_system::*, particle_system::*, visibility_system::*,
};
mod gamelog;
mod gui;
mod random_table;
mod saveload_system;
mod spawner;

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;

    let mut game_state = State { ecs: World::new() };
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<Monster>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<BlocksTile>();
    game_state.ecs.register::<CombatStats>();
    game_state.ecs.register::<SufferDamage>();
    game_state.ecs.register::<WantsToMelee>();
    game_state.ecs.register::<Item>();
    game_state.ecs.register::<ProvidesHealing>();
    game_state.ecs.register::<InBackpack>();
    game_state.ecs.register::<WantsToPickupItem>();
    game_state.ecs.register::<WantsToUseItem>();
    game_state.ecs.register::<WantsToDropItem>();
    game_state.ecs.register::<Consumable>();
    game_state.ecs.register::<Ranged>();
    game_state.ecs.register::<InflictsDamage>();
    game_state.ecs.register::<AreaOfEffect>();
    game_state.ecs.register::<Confusion>();
    game_state.ecs.register::<SimpleMarker<SerializeMe>>();
    game_state.ecs.register::<SerializationHelper>();
    game_state.ecs.register::<Equippable>();
    game_state.ecs.register::<Equipped>();
    game_state.ecs.register::<MeleePowerBonus>();
    game_state.ecs.register::<DefenseBonus>();
    game_state.ecs.register::<WantsToRemoveItem>();
    game_state.ecs.register::<ParticleLifetime>();
    game_state.ecs.register::<MagicMapper>();

    game_state
        .ecs
        .insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map = Map::new_map_rooms_and_corridors(1);
    let player_pos = map.rooms[0].center();

    let player_entity = spawner::player(&mut game_state.ecs, player_pos.x, player_pos.y);

    game_state.ecs.insert(RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut game_state.ecs, room, 1);
    }

    game_state.ecs.insert(map);
    game_state
        .ecs
        .insert(Point::new(player_pos.x, player_pos.y));
    game_state.ecs.insert(player_entity);
    game_state.ecs.insert(RunState::PreRun);
    game_state.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to the Dungeon".to_string()],
    });
    game_state.ecs.insert(ParticleBuilder::new());

    main_loop(context, game_state)
}

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        cull_dead_particles(&mut self.ecs, ctx);

        let mut new_run_state;
        {
            let run_state = self.ecs.fetch::<RunState>();
            new_run_state = *run_state;
        }

        match new_run_state {
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        new_run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    MainMenuResult::Selected { selected } => match selected {
                        MainMenuSelection::NewGame => new_run_state = RunState::PreRun,
                        MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            new_run_state = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        MainMenuSelection::Quit => ::std::process::exit(0),
                    },
                }
            }
            _ => {
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems(); // change to dispatcher!
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        new_run_state = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => new_run_state = RunState::MonsterTurn,
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            new_run_state = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert item");
                            new_run_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insetr item during dropping.");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: result.1,
                                },
                            )
                            .expect("Unable to insert item.");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        new_run_state = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    MainMenuResult::Selected { selected } => match selected {
                        MainMenuSelection::NewGame => new_run_state = RunState::PreRun,
                        MainMenuSelection::LoadGame => new_run_state = RunState::PreRun,
                        MainMenuSelection::Quit => ::std::process::exit(0),
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);

                new_run_state = RunState::MainMenu {
                    menu_selection: MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                new_run_state = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_run_state = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..MAPWIDTH {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row as usize == MAPHEIGHT - 1 {
                    new_run_state = RunState::MonsterTurn;
                } else {
                    new_run_state = RunState::MagicMapReveal { row: row + 1 };
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut map_index = MapIndexingSystem {};
        map_index.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);
        let mut particles = ParticleSpawnSystem {};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            let eq = equipped.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Failed to delete an Entity");
        }

        let worldmap;
        let current_depth;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            current_depth = worldmap_resource.depth;
            *worldmap_resource = Map::new_map_rooms_and_corridors(current_depth + 1);
            worldmap = worldmap_resource.clone();
        }

        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, current_depth + 1);
        }

        let player_pos = worldmap.rooms[0].center();
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_pos.x, player_pos.y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_pos.x;
            player_pos_comp.y = player_pos.y;
        }

        let mut viewshed_componens = self.ecs.write_component::<Viewshed>();
        let vs = viewshed_componens.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }

        let mut gamelog = self.ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("You have descended to the next level, and took a moment to heal".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Build a new map and place the player
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = Map::new_map_rooms_and_corridors(1);
            worldmap = worldmap_resource.clone();
        }

        // Spawn enemies
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, 1);
        }

        // Place the player and update resources
        let player_pos = worldmap.rooms[0].center();
        let player_entity = spawner::player(&mut self.ecs, player_pos.x, player_pos.y);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_pos.x, player_pos.y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player_entity;
        let player_pos_comp = position_components.get_mut(player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_pos.x;
            player_pos_comp.y = player_pos.y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
}
