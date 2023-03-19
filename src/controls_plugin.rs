use bevy::prelude::*;

#[derive(Resource)]
pub struct CurrentGamepad(pub Gamepad);

pub struct ControlsPlugin;
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(controls_system);
    }
}

fn controls_system(
    mut commands: Commands,
    current_gamepad: Option<Res<CurrentGamepad>>,
    mut gamepad_event_reader: EventReader<GamepadEvent>,
) {
    for event in gamepad_event_reader.iter() {
        let gamepad_id = event.gamepad;
        match &event.event_type {
            GamepadEventType::Connected(_) => {
                if current_gamepad.is_none() {
                    commands.insert_resource(CurrentGamepad(gamepad_id));
                }
            },
            GamepadEventType::Disconnected => {
                if let Some(CurrentGamepad(current_gamepad_id)) = current_gamepad.as_deref() {
                    if *current_gamepad_id == gamepad_id {
                        commands.remove_resource::<CurrentGamepad>();
                    }
                }
            },
            _ => {},
        }
    }
}
