use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::collections::{HashMap, HashSet};

use bevy_rapier2d::prelude::*;

// Bundles:

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
    pub active_events: ActiveEvents,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct LadderBundle {
    #[from_int_grid_cell]
    pub sensor_bundle: SensorBundle,
    pub climbable: Climbable,
}

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct WaterBundle {
    #[from_int_grid_cell]
    pub sensor_bundle: SensorBundle,
    pub water: Water,
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,

    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    pub climber: Climber,
    pub ground_detection: GroundDetection,

    // Animation components
    animation_state: AnimationState,

    // Build Items Component manually by using `impl From<&EntityInstance>`
    #[from_entity_instance]
    items: Items,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct SnakeBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

// Components

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Water;

#[derive(Component)]
pub struct PlayerAnimations {
    pub idle: benimator::Animation,
    pub walk: benimator::Animation,
    pub jump_prep: benimator::Animation,
    pub jump_up: benimator::Animation,
    pub jump_down: benimator::Animation,
    pub jump_land: benimator::Animation,
    pub hit: benimator::Animation,
    pub slash: benimator::Animation,
    pub punch: benimator::Animation,
    pub run: benimator::Animation,
    pub climb: benimator::Animation,
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Climbable;

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Resource, Default)]
pub struct GameTouches(pub Vec<Vec2>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct Climber {
    pub climbing: bool,
    pub intersecting_climbables: HashMap<Entity, GlobalTransform>,
}

#[derive(Clone, Component, Debug, Eq, Default, PartialEq)]
pub struct Items(pub Vec<String>);

#[derive(Clone, Default, Component)]
pub struct GroundDetection {
    pub on_ground: bool,
}

#[derive(Clone, Default, Component)]
pub struct AccelerationStat(pub f32);

#[derive(Clone, Default, Component)]
pub struct MaxSpeedStat(pub Vec2);

#[derive(Clone, Default, Component)]
pub struct JumpForceStat(pub f32);

#[derive(Clone, Default, Component)]
pub struct FakeGroundFrictionStat(pub f32);
