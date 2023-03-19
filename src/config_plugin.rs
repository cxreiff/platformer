use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::io::Cursor;
use winit::window::Icon;

use bevy::{prelude::*, window::PresentMode};
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy::input::common_conditions::input_toggle_active;
// use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
// use bevy::diagnostic::LogDiagnosticsPlugin;
#[cfg(debug_assertions)]
use {
    bevy_debug_text_overlay::{screen_print, OverlayPlugin},
    crate::GameState,
};

// world constants
pub const ASPECT_RATIO: f32 = 10. / 16.;
pub const WIDTH: f32 = 320.;
pub const HEIGHT: f32 = WIDTH * ASPECT_RATIO;
pub const CAMERA_SPEED: f32 = 0.04;

// physics constants
pub const PIXELS_PER_METER: f32 = 1.;
pub const GRAVITY: f32 = 2300.;
pub const PLAYER_SPEED: f32 = 230.;

// gameplay constants
pub const MAX_STAMINA: u32 = 1;

#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub struct DebugOptions {
    printed_info_enabled: bool,
}

pub struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 1 })
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        window: WindowDescriptor {
                            title: "bevy_trunk_template".to_string(),
                            canvas: Some("#bevy".to_owned()),
                            fit_canvas_to_parent: true,
                            present_mode: PresentMode::AutoVsync,
                            // mode: WindowMode::Fullscreen,
                            width: WIDTH,
                            height: HEIGHT,
                            ..default()
                        },
                        ..default()
                    })
                    .set(AssetPlugin {
                        watch_for_changes: true,
                        ..Default::default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .add_plugin(LdtkPlugin)
            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
                PIXELS_PER_METER,
            ))
            .insert_resource(RapierConfiguration {
                gravity: Vec2 { x: 0., y: -GRAVITY },
                ..Default::default()
            })
            .add_startup_system(window_icon_setup);

        #[cfg(debug_assertions)]
        {
            app.insert_resource(DebugOptions::default())
            .add_plugin(OverlayPlugin::default())
            .add_plugin(RapierDebugRenderPlugin::default().disabled())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            // .add_plugin(LogDiagnosticsPlugin::default())
            // .add_plugin(WorldInspectorPlugin::default().run_if(should_run))
            .add_system(debug_toggle_system)
            .add_system(debug_system);
        }
    }
}

fn window_icon_setup(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!("../assets/textures/app_icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

#[cfg(debug_assertions)]
fn debug_system(
    time: Res<Time>,
    debug_options: Res<DebugOptions>,
    windows: Res<Windows>,
    app_state: Res<State<GameState>>,
) {
    let current_time = time.elapsed_seconds();
    let at_interval = |t: f32| current_time % t < time.delta_seconds();
    if debug_options.printed_info_enabled && at_interval(0.25) {
        let window = windows.get_primary().unwrap();
        screen_print!(sec: 0.3, col: Color::CYAN, "game state: {:?}", app_state.current());
        if let Some(position) = window.cursor_position() {
            screen_print!(sec: 0.3, col: Color::CYAN, "cursor_position: {}", position);
        };
    }
}

#[cfg(debug_assertions)]
fn debug_toggle_system(
    input: Res<Input<KeyCode>>,
    mut debug_options: ResMut<DebugOptions>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if input.just_pressed(KeyCode::Key1) {
        debug_options.printed_info_enabled = !debug_options.printed_info_enabled;
    }
    if input.just_pressed(KeyCode::Key2) {
        rapier_debug.enabled = !rapier_debug.enabled;
    }
}

pub fn get_world_position(
    raw_position: Vec2,
    window: &Window,
    camera_transform: &GlobalTransform,
) -> Vec3 {
    let adjusted_position = Vec3::new(
        raw_position.x / window.width() * WIDTH,
        raw_position.y / window.height() * HEIGHT,
        0.,
    );

    *camera_transform * adjusted_position
}
