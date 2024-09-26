use spirv_std::glam::{Vec2, Vec3};

use crate::util;

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub t: f32,
    pub seed: Vec2,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, seed: Vec2) -> Self {
        Self {
            origin,
            direction,
            t: 0.0,
            seed,
        }
    }
}
