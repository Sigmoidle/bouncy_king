#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use bevy::{
    asset::AssetMetaCheck, prelude::*, render::camera::ScalingMode, window::WindowResolution,
};

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(GameTouches::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, touch_input)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                resolution: WindowResolution::new(960.0, 540.0),
                canvas: Some("#game".to_string()),
                ..default()
            }),
            ..default()
        }))
        .run();
}

const CAMERA_WORLD_SHAPE: Vec2 = Vec2 { x: 96.0, y: 54.0 };

#[derive(Resource, Default)]
struct GameTouches(Vec<Vec2>);

fn setup_camera(mut cmd: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: CAMERA_WORLD_SHAPE.x,
        min_height: CAMERA_WORLD_SHAPE.y,
    };
    cmd.spawn(camera_bundle);
}

fn touch_input(
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

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
