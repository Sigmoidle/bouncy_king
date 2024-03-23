use bevy::prelude::*;

// Window stuff
pub const BASE_RES: Vec2 = Vec2 {
    x: 1280.0,
    y: 720.0,
};

// LDTK stuff
pub enum CollideEnums {
    RedBrick = 1,
    BlueBrick,
    Water,
    Ladder,
}

// Physics engine stuff
pub const GRAVITY: f32 = -2000.0;
pub const PIXELS_PER_METER: f32 = 100.0;

// Game stuff

pub const DEFAULT_SPAWN: Vec3 = Vec3 {
    x: 200.0,
    y: -170.0,
    z: 10.0,
};
