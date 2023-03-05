// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::App;

use bevy_app::ConfigPlugin;
use bevy_app::GamePlugin;

fn main() {
    App::new()
        .add_plugin(ConfigPlugin)
        .add_plugin(GamePlugin)
        .run();
}
