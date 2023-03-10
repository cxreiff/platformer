use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{loading_plugin::AllAssets, GameState, HEIGHT, WIDTH};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::Index(0))
            .add_system_set(SystemSet::on_enter(GameState::Play).with_system(level_setup));
    }
}

fn level_setup(mut commands: Commands, levels: Res<AllAssets>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: levels.level0.clone(),
        transform: Transform {
            translation: Vec3 {
                x: -WIDTH / 2.,
                y: -HEIGHT / 2.,
                z: 0.,
            },
            scale: Vec3 {
                x: 2.75,
                y: 2.75,
                z: 1.,
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
