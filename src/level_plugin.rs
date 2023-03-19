use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{loading_plugin::AllAssets, GameState};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::Index(0))
            .insert_resource(LdtkSettings {
                set_clear_color: SetClearColor::No,
                level_background: LevelBackground::Nonexistent,
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                ..Default::default()
            })
            .add_system_set(SystemSet::on_enter(GameState::Play).with_system(level_setup));
    }
}

fn level_setup(mut commands: Commands, levels: Res<AllAssets>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: levels.level0.clone(),
        transform: Transform {
            ..Default::default()
        },
        ..Default::default()
    });
}
