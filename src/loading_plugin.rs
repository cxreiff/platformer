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
        app.add_loading_state(
            LoadingState::new(GameState::Load)
                .continue_to_state(GameState::Play)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec!["manifest.assets"])
                .with_collection::<AllAssets>(),
        )
        .add_state(GameState::Load);
    }
}
