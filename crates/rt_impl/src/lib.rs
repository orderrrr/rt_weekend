use std::f32::INFINITY;

use bytemuck::{Pod, Zeroable};

use hittable::{Hitable, HittableE, Interval};
use material::Material;
use ray::Ray;

use spirv_std::glam::{uvec2, vec2, vec3, UVec2, Vec3};

pub mod color;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod util;

use color::Saturate;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub aa_stages: u32,
    pub bounce_limit: i32,
}

fn rt(sc: &ShaderConstants, r: Ray, world: &HittableE) -> Vec3 {
    let mut r = r;
    let mut hit = world.hit(&r, Interval::new(0.0, INFINITY));
    let mut color = Vec3::splat(1.0);
    let mut iter = 0;

    loop {
        if iter > sc.bounce_limit {
            break;
        }

        match &hit {
            Some(h) => {
                let mat = h.material.scatter(&r, &h);
                match mat.ray {
                    Some(s) => {
                        color *= 1.0 / mat.attenuation;
                        hit = world.hit(&s, Interval::new(0.0001, INFINITY));
                        r = s;
                        iter += 1;
                    }
                    None => {
                        color *= 1.0 / mat.attenuation;
                        break;
                    }
                }
            }
            None => break,
        }
    }

    color = 1.0 / color;

    let rdu = r.direction.normalize();
    let a = 0.5 * (rdu.y + 1.0);
    return ((1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)) * color;
}

pub fn render_px(sc: &ShaderConstants, world: &HittableE, idx: UVec2) -> Vec3 {
    let time = 1.0; // right now we are not using time

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

        let ro = vec3(0.0, 0.0, 0.3);
        let rd = vec3(uv.x, uv.y, -1.5).normalize();

        let seed = util::hash22(uv + (i as f32) * (time % 100.));

        color += rt(sc, Ray::new(ro, rd, seed), &world).saturate();
    }

    linear_to_gamma((color / sc.aa_stages as f32).clamp(Vec3::splat(0.0), Vec3::splat(1.0)))
}

pub fn linear_to_gamma(splat: Vec3) -> Vec3 {
    vec3(
        linear_to_gamma_f32(splat.x),
        linear_to_gamma_f32(splat.y),
        linear_to_gamma_f32(splat.z),
    )
}

pub fn linear_to_gamma_f32(f: f32) -> f32 {
    if f > 0. {
        f.sqrt()
    } else {
        0.0
    }
}
