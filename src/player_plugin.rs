use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    config_plugin::{MAX_STAMINA, PLAYER_SPEED},
    controls_plugin::CurrentGamepad,
    wall_plugin::ContactDetection,
    GameState,
};

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(7.9, 8.0),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.8,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints: LockedAxes::ROTATION_LOCKED,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ControllerBundle {
    controller: KinematicCharacterController,
}

impl From<&EntityInstance> for ControllerBundle {
    fn from(entity_instance: &EntityInstance) -> ControllerBundle {
        match entity_instance.identifier.as_ref() {
            "Player" => ControllerBundle::default(),
            _ => ControllerBundle::default(),
        }
    }
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct LastSafeSpot(Vec3);

#[derive(Component, Clone, Deref, DerefMut)]
pub struct Stamina(u32);

impl Default for Stamina {
    fn default() -> Self {
        Self(MAX_STAMINA)
    }
}

#[derive(Component, Clone, Default)]
pub struct PlayerFlag;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_bundle: SpriteSheetBundle,

    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,

    #[from_entity_instance]
    #[bundle]
    controller: ControllerBundle,

    #[worldly]
    wordly: Worldly,

    contact_detection: ContactDetection,
    last_safe_spot: LastSafeSpot,
    stamina: Stamina,
    player_flag: PlayerFlag,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(
                (player_movement, update_safe_spot, check_out_of_level)
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

fn player_movement(
    input: Res<Input<KeyCode>>,
    buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    gamepad: Option<Res<CurrentGamepad>>,
    mut query: Query<
        (
            &mut Velocity,
            &ContactDetection,
            &mut Stamina,
        ),
        With<PlayerFlag>,
    >,
) {
    let mut jump_pressed = input.just_pressed(KeyCode::Up) || input.just_pressed(KeyCode::Space);
    let mut left_pressed = input.pressed(KeyCode::Left);
    let mut right_pressed = input.pressed(KeyCode::Right);
    let mut down_pressed = input.pressed(KeyCode::Down);
    let mut down_just_pressed = input.just_pressed(KeyCode::Down);
    let mut left_stick_x = 0.;
    if let Some(gp) = gamepad {
        let gamepad = gp.0;
        jump_pressed ^= buttons.just_pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::East,
        });
        left_pressed ^= buttons.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadLeft,
        });
        right_pressed ^= buttons.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadRight,
        });
        down_pressed ^= buttons.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadDown,
        });
        down_just_pressed ^= buttons.just_pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadDown,
        });
        if let Some(x) = axes.get(GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        }) {
            if x < -0.3 {
                left_stick_x = x;
                left_pressed = true;
            } else if x > 0.3 {
                left_stick_x = x;
                right_pressed = true;
            } else {
                left_stick_x = 0.;
            }
        }
    };

    if let Ok((mut velocity, contact_detection, mut stamina)) = query.get_single_mut()
    {
        let l = if left_pressed {
            if left_stick_x != 0. {
                (-left_stick_x * 1.3).min(1.)
            } else {
                1.
            }
        } else {
            0.
        };
        let r = if right_pressed {
            if left_stick_x != 0. {
                (left_stick_x * 1.3).min(1.)
            } else {
                1.
            }
        } else {
            0.
        };

        if r > 0. || l > 0. {
            if contact_detection.on_ground && down_pressed {
                velocity.linvel.x = (r - l).clamp(-0.5, 0.5) * PLAYER_SPEED;
            } else {
                velocity.linvel.x = (r - l).clamp(-1., 1.) * PLAYER_SPEED;
            }
        } else if !contact_detection.on_ground {
            velocity.linvel.x *= 0.97;
        }

        if (left_pressed && contact_detection.on_left)
            || (right_pressed && contact_detection.on_right)
        {
            velocity.linvel.y = velocity.linvel.y.max(-15.);
        }

        if jump_pressed {
            if contact_detection.on_ground {
                velocity.linvel.y = 500.;
            } else if contact_detection.on_left {
                velocity.linvel.y = 400.;
                if !input.pressed(KeyCode::Left) {
                    velocity.linvel.x = 300.;
                }
            } else if contact_detection.on_right {
                velocity.linvel.y = 400.;
                if !input.pressed(KeyCode::Right) {
                    velocity.linvel.x = -300.;
                }
            } else if **stamina > 0 {
                velocity.linvel.y = 400.;
                **stamina -= 1;
            }
        }

        if down_just_pressed && !contact_detection.on_ground {
            velocity.linvel.x = 0.;
            velocity.linvel.y = -800.;
        }

        if contact_detection.on_ground {
            **stamina = MAX_STAMINA;
        }
    }
}

pub fn update_safe_spot(
    mut contact_detectors_query: Query<(&ContactDetection, &mut LastSafeSpot, &Transform)>,
) {
    for (ContactDetection { is_stable, .. }, mut last_safe_spot, Transform { translation, .. }) in
        &mut contact_detectors_query
    {
        if *is_stable {
            **last_safe_spot = *translation;
        }
    }
}

fn check_out_of_level(
    mut query: Query<(&mut Transform, &mut Velocity, &LastSafeSpot), With<PlayerFlag>>,
) {
    for (mut transform, mut velocity, last_safe_spot) in &mut query {
        if transform.translation.y < -80. {
            transform.translation = **last_safe_spot;
            velocity.linvel = Vec2::new(0., 0.);
        }
    }
}
