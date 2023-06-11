use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::LdtkAsset;

use crate::GameState;

#[derive(AssetCollection, Resource)]
pub struct AllAssets {
    #[asset(key = "textures.kitty")]
    pub kitty: Handle<Image>,
    #[asset(key = "ldtk.level0")]
    pub level0: Handle<LdtkAsset>,
}

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
            )
            .add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
                GameState::Loading,
                "manifest.assets.ron",
            )
            .add_collection_to_loading_state::<_, AllAssets>(GameState::Loading);
    }
}
