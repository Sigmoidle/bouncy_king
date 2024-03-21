#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::type_complexity)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod components;
mod constants;
mod debug;
mod ldtk_spawning;
mod systems;

fn main() {
    App::new()
        // # Resources
        // - Touch-control
        .insert_resource(components::GameTouches::default())
        // - Bevy Engine settings
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(Msaa::Off)
        // - Physics engine settings
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, constants::GRAVITY),
            ..Default::default()
        })
        // - LDTK settings
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        // # LDTK settings
        // - Register entities
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::SnakeBundle>("Snake")
        // - Register "collide" int grids
        .register_ldtk_int_cell_for_layer::<components::WallBundle>(
            "Collide",
            constants::CollideEnums::RedBrick as i32,
        )
        .register_ldtk_int_cell_for_layer::<components::WallBundle>(
            "Collide",
            constants::CollideEnums::BlueBrick as i32,
        )
        .register_ldtk_int_cell_for_layer::<components::LadderBundle>(
            "Collide",
            constants::CollideEnums::Ladder as i32,
        )
        .register_ldtk_int_cell_for_layer::<components::WaterBundle>(
            "Collide",
            constants::CollideEnums::Water as i32,
        )
        // # Systems
        // - Startup systems
        .add_systems(Startup, (systems::setup, systems::setup_camera))
        // - Delayed startup systems (Due to the way LDTK loads stuff in)
        .add_systems(Update, ldtk_spawning::setup_player_components)
        // - Update systems
        .add_systems(
            Update,
            (
                systems::spawn_ground_sensor,
                systems::touch_input,
                systems::spawn_wall_collision,
                systems::player_movement,
                systems::update_on_ground,
                systems::update_ground_sensor_intersections,
                systems::update_climb_intersection_detection,
                systems::ignore_gravity_if_climbing,
                systems::update_player_animations,
                systems::update_level_selection,
                systems::camera_fit_inside_current_level,
                systems::clamp_velocity,
                systems::apply_fake_friction_while_climbing,
                systems::apply_fake_friction_on_ground,
                systems::snap_player_to_climbable,
                systems::update_climbing_status,
            ),
        )
        // # Plugins
        .add_plugins((
            // - Default bevy plugin
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(
                            constants::BASE_RES.x,
                            constants::BASE_RES.y,
                        ),
                        canvas: Some("#game".to_string()),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            // - LDTK
            LdtkPlugin,
            // - Physics engine
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(constants::PIXELS_PER_METER),
            debug::add_plugin,
        ))
        .run();
}
