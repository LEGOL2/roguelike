use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPWIDTH * MAPHEIGHT;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    // Create an empty map
    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
            tile_content: vec![Vec::new(); MAPCOUNT],
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, _idx: usize) -> bool {
        self.tiles[_idx] == TileType::Wall
    }

    fn get_available_exits(&self, _idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = _idx as i32 % self.width;
        let y = _idx as i32 / self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) {
            exits.push((_idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((_idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((_idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((_idx + w, 1.0))
        };

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((_idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((_idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((_idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((_idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, _idx1: usize, _idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(_idx1 % w, _idx1 / w);
        let p2 = Point::new(_idx2 % w, _idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub fn draw_map(map: &Map, ctx: &mut BTerm) {
    let mut x = 0;
    let mut y = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            let mut bg = RGB::from_f32(0., 0., 0.);
            match tile {
                TileType::Floor => {
                    glyph = to_cp437('·');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                }
                TileType::DownStairs => {
                    glyph = to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
            }
            if map.bloodstains.contains(&idx) {
                bg = RGB::from_f32(0.75, 0.0, 0.0);
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
                bg = RGB::from_f32(0.0, 0.0, 0.0);
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        x += 1;
        if x > MAPWIDTH as i32 - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
    if x < 1 || x > map.width - 2 as i32 {
        return 186;
    }

    if y < 1 || y > map.height - 2 as i32 {
        return 205;
    }

    let mut mask: u8 = 0;
    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,    // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
