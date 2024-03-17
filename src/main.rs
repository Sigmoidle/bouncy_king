#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod components;
mod systems;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(Msaa::Off)
        .insert_resource(components::GameTouches::default())
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::SnakeBundle>("Snake")
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::WallBundle>(2)
        .register_ldtk_int_cell::<components::LadderBundle>(4)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..Default::default()
        })
        .add_systems(
            Startup,
            (
                systems::setup.after(systems::setup_camera),
                systems::setup_camera,
            ),
        )
        .add_systems(
            Update,
            (
                systems::touch_input,
                systems::spawn_wall_collision,
                systems::movement,
                systems::spawn_ground_sensor,
                systems::update_on_ground,
                systems::ground_detection,
                systems::detect_climb_range,
                systems::ignore_gravity_if_climbing,
            ),
        )
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(BASE_RES.x, BASE_RES.y),
                        canvas: Some("#game".to_string()),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((
            LdtkPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
        ))
        .run();
}

const CAMERA_WORLD_SHAPE: Vec2 = Vec2 { x: 32.0, y: 18.0 };
const BASE_RES: Vec2 = Vec2 { x: 960.0, y: 540.0 };
