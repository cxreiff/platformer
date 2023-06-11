use bevy::prelude::*;
use bevy::input::gamepad::GamepadEvent;

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
        if let GamepadEvent::Connection(info) = event {
            if info.connected() && current_gamepad.is_none() {
                commands.insert_resource(CurrentGamepad(info.gamepad))
            }
            if info.disconnected() {
                if let Some(CurrentGamepad(current_gamepad_id)) = current_gamepad.as_deref() {
                    if *current_gamepad_id == info.gamepad {
                        commands.remove_resource::<CurrentGamepad>();
                    }
                }
            }
        }
    }
}
