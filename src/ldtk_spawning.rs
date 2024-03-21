use crate::components::{
    AccelerationStat, ColliderBundle, FakeGroundFrictionStat, Items, JumpForceStat, MaxSpeedStat,
    Player, PlayerAnimations, SensorBundle,
};
use crate::constants::CollideEnums;
use benimator::FrameRate;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
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
                collider: Collider::capsule_y(7., 7.),
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
                collider: Collider::cuboid(5., 5.),
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
                .insert(MaxSpeedStat(Vec2 { x: 150.0, y: 400.0 }))
                .insert(JumpForceStat(400.0))
                .insert(FakeGroundFrictionStat(-0.1));
        }
    }
}
