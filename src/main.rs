// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::App;

use platformer::ConfigPlugin;
use platformer::GamePlugin;

fn main() {
    App::new()
        .add_plugin(ConfigPlugin)
        .add_plugin(GamePlugin)
        .run();
}
