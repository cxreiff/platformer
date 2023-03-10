use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::GameState;

pub struct PlayerPlugin;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_bundle: SpriteSheetBundle,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .add_system_set(SystemSet::on_enter(GameState::Play).with_system(player_setup))
            .add_system_set(SystemSet::on_update(GameState::Play).with_system(player_movement));
    }
}

fn player_setup() {}

fn player_movement() {}
