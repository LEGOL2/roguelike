use bracket_lib::prelude::*;

embedded_resource!(DUNGEON_ENTRANCE, "./resources/entrance.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> Self {
        link_resource!(DUNGEON_ENTRANCE, "./resources/entrance.xp");

        RexAssets {
            menu: XpFile::from_resource("./resources/entrance.xp").unwrap(),
        }
    }
}
