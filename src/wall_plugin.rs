use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct WallPlugin;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell::<WallBundle>(1)
            .add_system(spawn_wall_collision)
            .add_system(spawn_sensors)
            .add_system(contact_detection)
            .add_system(update_contact_detectors);
    }
}

#[derive(Component)]
pub struct ContactSensor {
    pub detecting_entity: Entity,
    pub intersecting_entities: HashSet<Entity>,
}

#[derive(Component)]
pub struct ContactSensorLeft;

#[derive(Component)]
pub struct ContactSensorRight;

#[derive(Component)]
pub struct ContactSensorGround;

#[derive(Component)]
pub struct ContactSensorStableLeft;

#[derive(Component)]
pub struct ContactSensorStableRight;

#[derive(Component, Clone, Default)]
pub struct ContactDetection {
    pub on_left: bool,
    pub on_right: bool,
    pub on_ground: bool,
    pub stable_left: bool,
    pub stable_right: bool,
    pub is_stable: bool,
}

// COPIED FROM bevy_ecs_ldtk EXAMPLE

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

pub fn spawn_sensors(
    mut commands: Commands,
    contact_detectors: Query<(Entity, &Collider), Added<ContactDetection>>,
) {
    for (entity, collider) in &contact_detectors {
        if let Some(cuboid) = collider.as_cuboid() {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid.half_extents();

            let sensor_collider_left = Collider::cuboid(half_extents_x / 2., 3.9);
            let sensor_translation_left = Vec3::new(-half_extents_x, 0., 0.);

            let sensor_collider_right = Collider::cuboid(half_extents_x / 2., 3.9);
            let sensor_translation_right = Vec3::new(half_extents_x, 0., 0.);

            let sensor_collider_ground = Collider::cuboid(half_extents_x - 1., 2.);
            let sensor_translation_ground = Vec3::new(0., -half_extents_y, 0.);

            let sensor_collider_stable_left = Collider::cuboid(half_extents_x / 8., 2.);
            let sensor_translatiom_stable_left =
                Vec3::new(-half_extents_x + 1., -half_extents_y, 0.);

            let sensor_collider_stable_right = Collider::cuboid(half_extents_x / 8., 2.);
            let sensor_translatiom_stable_right =
                Vec3::new(half_extents_x - 1., -half_extents_y, 0.);

            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(sensor_collider_left)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation_left))
                    .insert(GlobalTransform::default())
                    .insert(ContactSensorLeft)
                    .insert(ContactSensor {
                        detecting_entity: entity,
                        intersecting_entities: HashSet::new(),
                    });
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(sensor_collider_right)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation_right))
                    .insert(GlobalTransform::default())
                    .insert(ContactSensorRight)
                    .insert(ContactSensor {
                        detecting_entity: entity,
                        intersecting_entities: HashSet::new(),
                    });
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(sensor_collider_ground)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation_ground))
                    .insert(GlobalTransform::default())
                    .insert(ContactSensorGround)
                    .insert(ContactSensor {
                        detecting_entity: entity,
                        intersecting_entities: HashSet::new(),
                    });
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(sensor_collider_stable_left)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translatiom_stable_left))
                    .insert(GlobalTransform::default())
                    .insert(ContactSensorStableLeft)
                    .insert(ContactSensor {
                        detecting_entity: entity,
                        intersecting_entities: HashSet::new(),
                    });
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(sensor_collider_stable_right)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translatiom_stable_right))
                    .insert(GlobalTransform::default())
                    .insert(ContactSensorStableRight)
                    .insert(ContactSensor {
                        detecting_entity: entity,
                        intersecting_entities: HashSet::new(),
                    });
            });
        }
    }
}

pub fn contact_detection(
    mut contact_sensors: Query<&mut ContactSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<With<Collider>, Without<Sensor>>,
) {
    for collision_event in collisions.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = contact_sensors.get_mut(*e2) {
                        sensor.intersecting_entities.insert(*e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = contact_sensors.get_mut(*e1) {
                        sensor.intersecting_entities.insert(*e2);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = contact_sensors.get_mut(*e2) {
                        sensor.intersecting_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = contact_sensors.get_mut(*e1) {
                        sensor.intersecting_entities.remove(e2);
                    }
                }
            }
        }
    }
}

pub fn update_contact_detectors(
    mut contact_detectors: Query<&mut ContactDetection>,
    ground_sensors: Query<
        (
            &ContactSensor,
            Option<&ContactSensorLeft>,
            Option<&ContactSensorRight>,
            Option<&ContactSensorGround>,
            Option<&ContactSensorStableLeft>,
            Option<&ContactSensorStableRight>,
        ),
        Changed<ContactSensor>,
    >,
) {
    for (sensor, left, right, ground, stable_left, stable_right) in &ground_sensors {
        if let Ok(mut contact_detection) = contact_detectors.get_mut(sensor.detecting_entity) {
            if left.is_some() {
                contact_detection.on_left = !sensor.intersecting_entities.is_empty();
            }
            if right.is_some() {
                contact_detection.on_right = !sensor.intersecting_entities.is_empty();
            }
            if ground.is_some() {
                contact_detection.on_ground = !sensor.intersecting_entities.is_empty();
            }
            if stable_left.is_some() {
                contact_detection.stable_left = !sensor.intersecting_entities.is_empty();
                contact_detection.is_stable =
                    contact_detection.stable_left && contact_detection.stable_right;
            }
            if stable_right.is_some() {
                contact_detection.stable_right = !sensor.intersecting_entities.is_empty();
                contact_detection.is_stable =
                    contact_detection.stable_left && contact_detection.stable_right;
            }
        }
    }
}
