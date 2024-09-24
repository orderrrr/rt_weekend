use std::f32::INFINITY;

use bytemuck::{Pod, Zeroable};

use hittable::{Hitable, Interval};
use ray::Ray;

use spirv_std::glam::{uvec2, vec2, vec3, UVec2, Vec3};

pub mod color;
pub mod hittable;
pub mod ray;
pub mod util;

use color::Saturate;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub aa_stages: u32,
}

fn rt(r: Ray, world: &dyn Hitable) -> Vec3 {
    let hit = world.hit(&r, Interval::new(0.0, INFINITY));

    match hit {
        Some(h) => {
            return 0.5 * (h.normal + 1.0);
        }
        None => {
            let rdu = r.direction.normalize();
            let a = 0.5 * (rdu.y + 1.0);
            (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)
        }
    }
}

pub fn render_px(sc: &ShaderConstants, world: &dyn Hitable, idx: UVec2) -> Vec3 {
    let p = idx.as_vec2();

    let mut color = Vec3::splat(0.0);

    for i in 0..sc.aa_stages {
        // calc uv and flipping uv.y
        let mut uv =
            ((2.0 * p - uvec2(sc.width, sc.height).as_vec2()) / sc.height as f32) * vec2(1.0, -1.);

        let offset = i as f32 * idx.as_vec2();
        let position = vec2(
            util::rand_f32(offset.x) - 0.5,
            util::rand_f32(offset.y) - 0.5,
        );

        uv += position * 0.005;

        let ro = vec3(0.0, 0.0, 0.0);
        let rd = vec3(uv.x, uv.y, -1.5).normalize();

        color += rt(Ray::new(ro, rd), world).saturate();
    }

    (color / sc.aa_stages as f32).clamp(Vec3::splat(0.0), Vec3::splat(1.0))
}
