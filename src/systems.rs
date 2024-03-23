use crate::components::{
    AccelerationStat, AnimationState, Climbable, Climber, FakeGroundFrictionStat, GameTouches,
    GroundDetection, GroundSensor, JumpForceStat, MaxSpeedStat, Player, PlayerAnimations, Wall,
    Water,
};
use crate::constants;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("world.ldtk"),
        ..Default::default()
    });
}

pub fn setup_camera(mut cmd: Commands) {
    cmd.spawn(Camera2dBundle::default());
}

pub fn touch_input(
    touches: Res<Touches>,
    mut game_touches: ResMut<GameTouches>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    game_touches.0 = touches
        .iter()
        .filter_map(|t| camera.viewport_to_world_2d(camera_transform, t.position()))
        .collect::<Vec<Vec2>>();
}

pub fn update_player_animations(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Sprite,
            &Velocity,
            &Climber,
            &GroundDetection,
            &mut AnimationState,
            &mut TextureAtlas,
            &PlayerAnimations,
        ),
        With<Player>,
    >,
) {
    for (mut sprite, velocity, climbing, ground, mut player, mut atlas, animations) in &mut query {
        let mut animation = &animations.idle;
        let mut update_animation = true;

        if velocity.linvel.x < -15.0 {
            sprite.flip_x = true;
        } else if velocity.linvel.x > 15.0 {
            sprite.flip_x = false;
        }
        if ground.on_ground {
            if velocity.linvel.x < -15.0 || velocity.linvel.x > 15.0 {
                animation = &animations.walk;
            }
        } else if climbing.climbing {
            if velocity.linvel.y == 0.0 {
                animation = &animations.climb;
                update_animation = false;
            } else {
                animation = &animations.climb;
            }
        } else if !ground.on_ground {
            if velocity.linvel.y > 0.0 {
                animation = &animations.jump_up;
            } else if velocity.linvel.y <= 0.0 {
                animation = &animations.jump_down;
            }
        }

        if update_animation {
            player.update(animation, time.delta());
        } else {
            player.update(animation, Duration::ZERO);
        }

        atlas.index = player.frame_index();
    }
}

pub fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &AccelerationStat,
            &JumpForceStat,
            &mut Velocity,
            &Climber,
            &GroundDetection,
        ),
        With<Player>,
    >,
) {
    for (acceleration_stat, jump_force_stat, mut velocity, climber, ground_detection) in &mut query
    {
        let right = if input.pressed(KeyCode::KeyD) { 1. } else { 0. };
        let left = if input.pressed(KeyCode::KeyA) { 1. } else { 0. };

        velocity.linvel.x +=
            (right - left) * acceleration_stat.0 + 0.5 * ((right - left) * acceleration_stat.0);

        if climber.climbing {
            let up = if input.pressed(KeyCode::KeyW) { 1. } else { 0. };
            let down = if input.pressed(KeyCode::KeyS) { 1. } else { 0. };
            velocity.linvel.y = (up - down) * 150.;
        }

        if input.just_pressed(KeyCode::Space) && (ground_detection.on_ground || climber.climbing) {
            velocity.linvel.y = jump_force_stat.0;
        }
    }
}

pub fn update_climbing_status(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Climber, With<Player>>,
) {
    for mut climber in &mut query {
        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::KeyS) {
            climber.climbing = true;
        } else if input.pressed(KeyCode::Space) {
            climber.climbing = false;
        }
    }
}

pub fn snap_player_to_climbable(
    mut query: Query<(&mut Transform, &mut Velocity, &Climber), With<Player>>,
) {
    for (mut transform, mut velocity, climber) in &mut query {
        if climber.climbing && velocity.linvel.y != 0.0 {
            velocity.linvel.x = 0.0;
            let climb_x_location = climber
                .intersecting_climbables
                .iter()
                .next()
                .map(|a| a.1.translation().x + 0.4 * a.1.compute_transform().scale.x);
            if let Some(x) = climb_x_location {
                transform.translation.x = x + 0.4 * transform.scale.x;
            }
        }
    }
}

pub fn apply_fake_friction_while_climbing(
    mut query: Query<(&FakeGroundFrictionStat, &mut Velocity, &Climber)>,
) {
    for (fake_friction, mut velocity, climber) in &mut query {
        if climber.climbing {
            velocity.linvel.x += velocity.linvel.x * fake_friction.0;
        }
    }
}

pub fn apply_fake_friction_on_ground(
    mut query: Query<(&FakeGroundFrictionStat, &mut Velocity, &GroundDetection)>,
) {
    for (fake_friction, mut velocity, ground_detection) in &mut query {
        if ground_detection.on_ground {
            velocity.linvel.x += velocity.linvel.x * fake_friction.0;
        }
    }
}

pub fn clamp_velocity(mut query: Query<(&MaxSpeedStat, &mut Velocity)>) {
    for (max_speed, mut velocity) in &mut query {
        velocity.linvel.x = velocity.linvel.x.clamp(-max_speed.0.x, max_speed.0.x);
        velocity.linvel.y = velocity.linvel.y.clamp(-max_speed.0.y, max_speed.0.y);
    }
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
    for (entity, shape) in &detect_ground_for {
        if let Some(cuboid) = shape.as_cuboid() {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid.half_extents();

            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

            let sensor_translation = Vec3::new(0., -half_extents_y, 0.);
            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(ActiveCollisionTypes::all())
                    .insert(detector_shape)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation))
                    .insert(GlobalTransform::default())
                    .insert(GroundSensor {
                        ground_detection_entity: entity,
                        intersecting_ground_entities: HashSet::new(),
                    });
            });
        } else if let Some(round) = shape.as_capsule() {
            let half_extents_x = round.radius();
            let half_extents_y = round.half_height() + half_extents_x;

            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

            let sensor_translation = Vec3::new(0., -half_extents_y, 0.);
            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(ActiveCollisionTypes::all())
                    .insert(detector_shape)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation))
                    .insert(GlobalTransform::default())
                    .insert(GroundSensor {
                        ground_detection_entity: entity,
                        intersecting_ground_entities: HashSet::new(),
                    });
            });
        }
    }
}

pub fn update_on_ground(
    mut ground_detectors: Query<&mut GroundDetection>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            ground_detection.on_ground = !sensor.intersecting_ground_entities.is_empty();
        }
    }
}

pub fn update_ground_sensor_intersections(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<Entity, (With<Collider>, Without<Sensor>)>,
) {
    for collision_event in collisions.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(*e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(*e2);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.remove(e2);
                    }
                }
            }
        }
    }
}

pub fn update_climb_intersection_detection(
    mut climbers: Query<&mut Climber>,
    climbables: Query<(Entity, &GlobalTransform), With<Climbable>>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for collision in collisions.read() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b, _) => {
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_a), climbables.get(*collider_b))
                {
                    climber
                        .intersecting_climbables
                        .insert(climbable.0, *climbable.1);
                }
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_b), climbables.get(*collider_a))
                {
                    climber
                        .intersecting_climbables
                        .insert(climbable.0, *climbable.1);
                };
            }
            CollisionEvent::Stopped(collider_a, collider_b, _) => {
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_a), climbables.get(*collider_b))
                {
                    climber.intersecting_climbables.remove(&climbable.0);
                }

                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_b), climbables.get(*collider_a))
                {
                    climber.intersecting_climbables.remove(&climbable.0);
                }
            }
        }
    }
}

pub fn check_touched_water(
    mut player: Query<&mut Transform, With<Player>>,
    waters: Query<(Entity, &GlobalTransform), With<Water>>,
    mut collisions: EventReader<CollisionEvent>,
    mut level_selection: ResMut<LevelSelection>,
) {
    for collision in collisions.read() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b, _) => {
                if let (Ok(mut player), Ok(_)) =
                    (player.get_mut(*collider_a), waters.get(*collider_b))
                {
                    player.translation = constants::DEFAULT_SPAWN;
                    *level_selection = LevelSelection::index(0);
                }
                if let (Ok(mut player), Ok(_)) =
                    (player.get_mut(*collider_b), waters.get(*collider_a))
                {
                    player.translation = constants::DEFAULT_SPAWN;
                    *level_selection = LevelSelection::index(0);
                }
            }
            CollisionEvent::Stopped(collider_a, collider_b, _) => {}
        }
    }
}

pub fn ignore_gravity_if_climbing(
    mut query: Query<(&Climber, &mut GravityScale), Changed<Climber>>,
) {
    for (climber, mut gravity_scale) in &mut query {
        if climber.climbing {
            gravity_scale.0 = 0.0;
        } else {
            gravity_scale.0 = 1.0;
        }
    }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a `ColliderBundle` in to the `WallBundle`,
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
/// # Panics
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    level_assets: Res<Assets<LdtkExternalLevel>>,
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

    wall_query.iter().for_each(|(&grid_coords, parent)| {
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
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_parent()
                    .get_external_level_by_iid(&level_assets, &level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..=width {
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
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default())
                            .insert(ActiveEvents::COLLISION_EVENTS);
                    }
                });
            }
        });
    }
}

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<OrthographicProjection>, Without<Player>)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    window: Query<&Window>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        // This can be optimised
        let window = window.single();
        let width = window.resolution.width();
        let height = window.resolution.height();
        let aspect_ratio = width / height;

        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_iid) in &level_query {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .get_raw_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            if level_selection.is_match(&LevelIndices::default(), level) {
                let level_ratio = level.px_wid as f32 / level.px_hei as f32;
                orthographic_projection.viewport_origin = Vec2::ZERO;
                if level_ratio > aspect_ratio {
                    // level is wider than the screen
                    let height = (level.px_hei as f32 / 9.).round() * 9.;
                    let width = height * aspect_ratio;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.x =
                        (player_translation.x - level_transform.translation.x - width / 2.)
                            .clamp(0., level.px_wid as f32 - width);
                    camera_transform.translation.y = 0.;
                } else {
                    // level is taller than the screen
                    let width = (level.px_wid as f32 / 16.).round() * 16.;
                    let height = width / aspect_ratio;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.y =
                        (player_translation.y - level_transform.translation.y - height / 2.)
                            .clamp(0., level.px_hei as f32 - height);
                    camera_transform.translation.x = 0.;
                }

                camera_transform.translation.x += level_transform.translation.x;
                camera_transform.translation.y += level_transform.translation.y;
            }
        }
    }
}

pub fn update_level_selection(
    level_query: Query<(&LevelIid, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for (level_iid, level_transform) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        let level_bounds = Rect {
            min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
            max: Vec2::new(
                level_transform.translation.x + level.px_wid as f32,
                level_transform.translation.y + level.px_hei as f32,
            ),
        };
        for player_transform in &player_query {
            if player_transform.translation.x < level_bounds.max.x
                && player_transform.translation.x > level_bounds.min.x
                && player_transform.translation.y < level_bounds.max.y
                && player_transform.translation.y > level_bounds.min.y
            {
                *level_selection = LevelSelection::iid(level.iid.clone());
            }
        }
    }
}
