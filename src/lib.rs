#![allow(clippy::type_complexity)]

use bevy::prelude::*;

mod config_plugin;
mod camera_plugin;
mod controls_plugin;
mod level_plugin;
mod loading_plugin;
mod player_plugin;
mod wall_plugin;

pub use config_plugin::{
    get_world_position, ConfigPlugin, ASPECT_RATIO, HEIGHT, WIDTH,
};
pub use loading_plugin::{LoadingPlugin, AllAssets};
use controls_plugin::ControlsPlugin;
use camera_plugin::CameraPlugin;
use level_plugin::LevelPlugin;
use player_plugin::PlayerPlugin;
use wall_plugin::WallPlugin;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(LoadingPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(ControlsPlugin)
            .add_plugin(LevelPlugin)
            .add_plugin(WallPlugin)
            .add_plugin(PlayerPlugin);
    }
}
