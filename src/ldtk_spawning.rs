use crate::components::{
    AccelerationStat, CanDie, ColliderBundle, Enemy, FakeGroundFrictionStat, IsLdtkEntity, Items,
    JumpForceStat, MaxSpeedStat, Patrol, PatrolAnimation, Player, PlayerAnimations, SensorBundle,
};
use crate::constants::CollideEnums;
use benimator::FrameRate;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::utils::ldtk_pixel_coords_to_translation_pivoted;
use bevy_rapier2d::prelude::*;

// Spawn sensors for int-grid from LDTK
impl From<IntGridCell> for SensorBundle {
    fn from(int_grid_cell: IntGridCell) -> SensorBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        // ladder
        if int_grid_cell.value == CollideEnums::Ladder as i32 {
            SensorBundle {
                collider: Collider::cuboid(5., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            }
        }
        // Water
        else if int_grid_cell.value == CollideEnums::Water as i32 {
            SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            }
        } else {
            SensorBundle::default()
        }
    }
}

// Spawn collider bundles for entities from LDTK
impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::capsule_y(2., 6.),
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                ..Default::default()
            },
            "Snake" => ColliderBundle {
                collider: Collider::cuboid(4., 4.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

// Spawn items component for entities from LDTK
impl From<&EntityInstance> for Items {
    fn from(entity_instance: &EntityInstance) -> Self {
        Items(
            entity_instance
                .iter_enums_field("Items")
                .expect("items field should be correctly typed")
                .cloned()
                .collect(),
        )
    }
}

impl LdtkEntity for Patrol {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Patrol {
        let mut points = Vec::new();
        points.push(
            ldtk_pixel_coords_to_translation_pivoted(
                entity_instance.px,
                layer_instance.c_hei * layer_instance.grid_size,
                IVec2::new(entity_instance.width, entity_instance.height),
                entity_instance.pivot,
            ) - Vec2 { x: 0.0, y: 8.0 },
        );

        let ldtk_patrol_points = entity_instance
            .iter_points_field("patrol")
            .expect("patrol field should be correclty typed");

        for ldtk_point in ldtk_patrol_points {
            // The +1 is necessary here due to the pivot of the entities in the sample
            // file.
            // The patrols set up in the file look flat and grounded,
            // but technically they're not if you consider the pivot,
            // which is at the bottom-center for the skulls.
            let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 0.5))
                * Vec2::splat(layer_instance.grid_size as f32);

            points.push(
                ldtk_pixel_coords_to_translation_pivoted(
                    pixel_coords.as_ivec2(),
                    layer_instance.c_hei * layer_instance.grid_size,
                    IVec2::new(entity_instance.width, entity_instance.height),
                    entity_instance.pivot,
                ) - Vec2 { x: 0.0, y: 8.0 },
            );
        }

        Patrol {
            points,
            index: 1,
            forward: true,
        }
    }
}

pub fn setup_player_components(mut cmd: Commands, query: Query<Entity, Added<Player>>) {
    let o = 22; // animation index offset in the sprite sheet
    let player_animations = PlayerAnimations {
        idle: benimator::Animation::from_indices((o + 12)..=(o + 13), FrameRate::from_fps(1.5)),
        walk: benimator::Animation::from_indices((o + 1)..=(o + 4), FrameRate::from_fps(12.0)),
        jump_prep: benimator::Animation::from_indices((o + 5)..=(o + 5), FrameRate::from_fps(12.0)),
        jump_up: benimator::Animation::from_indices((o + 6)..=(o + 6), FrameRate::from_fps(12.0)),
        jump_down: benimator::Animation::from_indices((o + 7)..=(o + 7), FrameRate::from_fps(12.0)),
        jump_land: benimator::Animation::from_indices((o + 8)..=(o + 8), FrameRate::from_fps(12.0)),
        hit: benimator::Animation::from_indices((o + 9)..(o + 10), FrameRate::from_fps(12.0)),
        slash: benimator::Animation::from_indices(
            [(o + 12), (o + 11), (o + 12), (o + 13)],
            FrameRate::from_fps(12.0),
        ),
        punch: benimator::Animation::from_indices([(o + 14), (o + 12)], FrameRate::from_fps(12.0)),
        run: benimator::Animation::from_indices((o + 15)..=(o + 18), FrameRate::from_fps(12.0)),
        climb: benimator::Animation::from_indices((o + 19)..=(o + 22), FrameRate::from_fps(12.0)),
    };

    if let Ok(entity) = query.get_single() {
        if let Some(mut entity_command) = cmd.get_entity(entity) {
            entity_command
                .insert(player_animations)
                .insert(ActiveCollisionTypes::all())
                .insert(AccelerationStat(15.0))
                .insert(MaxSpeedStat(Vec2 { x: 100.0, y: 400.0 }))
                .insert(JumpForceStat(400.0))
                .insert(FakeGroundFrictionStat(-0.1))
                .insert(CanDie {
                    is_dead: false,
                    dead_animation_timer: Timer::from_seconds(1.5, TimerMode::Once),
                });
        }
    }
}

pub fn add_patrol_animation_enemy(
    mut cmd: Commands,
    query: Query<Entity, (Added<Enemy>, Added<Patrol>)>,
) {
    let o = 68;
    let partrol_animation = PatrolAnimation(benimator::Animation::from_indices(
        (o + 1)..=(o + 4),
        FrameRate::from_fps(4.0),
    ));
    for entity in &query {
        if let Some(mut entity_command) = cmd.get_entity(entity) {
            entity_command.insert(partrol_animation.clone());
        }
    }
}

pub fn fix_sprite_translation(mut query: Query<&mut Sprite, Added<IsLdtkEntity>>) {
    for mut sprite in &mut query {
        sprite.anchor = Anchor::Custom(Vec2 { x: 0.0, y: -0.2 });
    }
}

pub fn fix_enemy_hitbox(mut query: Query<&mut Transform, Added<Enemy>>) {
    for mut transform in &mut query {
        transform.translation -= Vec3 {
            x: 0.0,
            y: 8.0,
            z: 0.0,
        }
    }
}
