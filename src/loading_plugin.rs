use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::GameState;

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(key = "textures.kitty")]
    pub kitty: Handle<Image>,
}

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Load)
                .continue_to_state(GameState::Play)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec!["manifest.assets"])
                .with_collection::<TextureAssets>(),
        )
        .add_state(GameState::Load);
    }
}
