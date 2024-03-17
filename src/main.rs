#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use bevy::{
    asset::AssetMetaCheck, prelude::*, render::camera::ScalingMode, window::WindowResolution,
};
use bevy_ecs_ldtk::prelude::*;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(GameTouches::default())
        .insert_resource(LevelSelection::index(0))
        .insert_resource(Msaa::Off)
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<SnakeBundle>("Snake")
        .add_systems(Startup, (setup.after(setup_camera), setup_camera))
        .add_systems(Update, touch_input)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        resolution: WindowResolution::new(960.0, 540.0),
                        canvas: Some("#game".to_string()),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(LdtkPlugin)
        .run();
}

const CAMERA_WORLD_SHAPE: Vec2 = Vec2 { x: 32.0, y: 18.0 };

#[derive(Resource, Default)]
struct GameTouches(Vec<Vec2>);

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Default, Bundle, LdtkEntity)]
struct SnakeBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("world.ldtk"),
        ..Default::default()
    });
}

fn setup_camera(mut cmd: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: CAMERA_WORLD_SHAPE.x,
        min_height: CAMERA_WORLD_SHAPE.y,
    };
    camera_bundle.transform.translation.x += 960.0 / 4.0;
    camera_bundle.transform.translation.y += 540.0 / 4.0;
    camera_bundle.projection.scale = 15.0;
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
