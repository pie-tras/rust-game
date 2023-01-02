use bevy::prelude::*;

mod tilemap;

use tilemap::TileMapPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(TileMapPlugin)
        .run();
}