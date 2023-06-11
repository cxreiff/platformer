use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::{config_plugin::CAMERA_SPEED, player_plugin::PlayerFlag, HEIGHT, WIDTH};

#[derive(Component)]
pub struct CameraFlag;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(camera_setup)
            .add_system(camera_follow_system);
    }
}

fn camera_setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedHorizontal(WIDTH),
                viewport_origin: Vec2::new(0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(CameraFlag);
}

fn camera_follow_system(
    player_query: Query<&Transform, With<PlayerFlag>>,
    mut camera_query: Query<&mut Transform, (With<CameraFlag>, Without<PlayerFlag>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) =
        (camera_query.get_single_mut(), player_query.get_single())
    {
        let diff = player_transform.translation + Vec3::new(-WIDTH / 2., -HEIGHT / 2., 0.)
            - camera_transform.translation;
        camera_transform.translation += Vec3::new(diff.x * CAMERA_SPEED, diff.y * CAMERA_SPEED, 0.);
    }
}
