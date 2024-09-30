use std::f32::{consts::PI, INFINITY};

use bytemuck::{Pod, Zeroable};

use hittable::{Hitable, HittableE, Interval, Sphere};
use material::{DialetricMaterial, LambertianMaterial, Material, MaterialE, MetalMaterial};
use ray::Ray;

use spirv_std::glam::{mat3, uvec2, vec2, vec3, vec4, Mat3, UVec2, Vec3, Vec4, Vec4Swizzles};
use util::{linear_to_gamma, linear_to_gamma_f32};

pub mod color;
pub mod depth;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod util;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub aa_stages: u32,
    pub bounce_limit: i32,
    pub focus_point: f32,
}

fn rt(sc: &ShaderConstants, r: Ray, world: &HittableE) -> Vec4 {
    let mut r = r;
    let mut hit = world.hit(&r, Interval::new(0.0, INFINITY));
    let mut color = Vec3::splat(1.0);

    let mut d = 100.0;

    // first hit we should provide the distance to the camera.
    match &hit {
        Some(h) => {
            d = r.origin.distance(h.position);
        }
        None => (),
    }

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
    color = ((1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0)) * color;
    vec4(color.x, color.y, color.z, d)
}

pub fn set_camera(ro: Vec3, ta: Vec3, cr: f32) -> Mat3 {
    let cw = (ta - ro).normalize();
    let cp = vec3(cr.sin(), cr.cos(), 0.0);
    let cu = cw.cross(cp).normalize();
    let cv = cu.cross(cw);
    mat3(cu, cv, cw)
}

pub fn render_pass_one(sc: &ShaderConstants, world: &HittableE, idx: UVec2) -> Vec4 {
    let time = 1.0; // right now we are not using time

    let p = idx.as_vec2();

    let mut color = Vec4::splat(0.0);

    // camera
    let ta = vec3(0.0, 0.0, -1.0);
    let ro = vec3(-2.0, 1.0, 1.0);
    let cam = set_camera(ro, ta, 0.0);

    for i in 0..sc.aa_stages {
        // calc uv and flipping uv.y
        let mut uv =
            ((2.0 * p - uvec2(sc.width, sc.height).as_vec2()) / sc.height as f32) * vec2(1.0, -1.);

        let offset = i as f32 * idx.as_vec2();
        let position = util::hash22(offset) - 0.5;

        uv += position * 0.005;

        let focal_length = 4.0;
        let rd = cam * vec3(uv.x, uv.y, focal_length).normalize();

        let seed = util::hash22(uv + (i as f32) * (time % 100.));

        color += rt(sc, Ray::new(ro, rd, seed), &world);
    }

    color / sc.aa_stages as f32
}

pub fn describe_scene() -> HittableE {
    let mat_ground = MaterialE::Lambertian(LambertianMaterial::new(vec3(0.8, 0.8, 0.0)));
    let mat_center = MaterialE::Lambertian(LambertianMaterial::new(vec3(0.1, 0.2, 0.5)));

    let mat_left = MaterialE::Dialetric(DialetricMaterial::new(vec3(1.0, 1.0, 1.0), 1.5));
    let mat_l_bubble = MaterialE::Dialetric(DialetricMaterial::new(vec3(1.0, 1.0, 1.0), 1.0 / 1.5));

    let mat_right = MaterialE::Metal(MetalMaterial::new(vec3(0.8, 0.6, 0.2), 0.8));

    HittableE::List(vec![
        HittableE::Sphere(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, mat_ground)),
        HittableE::Sphere(Sphere::new(Vec3::new(0.0, 0.0, -1.2), 0.5, mat_center)),
        HittableE::Sphere(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, mat_left)),
        HittableE::Sphere(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.4, mat_l_bubble)),
        HittableE::Sphere(Sphere::new(Vec3::new(1.0, 0.0, -1.0), 0.5, mat_right)),
    ])
}

pub fn describe_scene2() -> HittableE {
    let r = (PI / 4.0).cos();

    let mat_left = MaterialE::Lambertian(LambertianMaterial::new(vec3(0.0, 0.0, 1.0)));
    let mat_right = MaterialE::Lambertian(LambertianMaterial::new(vec3(1.0, 0.0, 0.0)));

    HittableE::List(vec![
        HittableE::Sphere(Sphere::new(Vec3::new(-r, 0.0, -1.0), r, mat_left)),
        HittableE::Sphere(Sphere::new(Vec3::new(r, 0.0, -1.0), r, mat_right)),
    ])
}

pub fn describe_scene3() -> HittableE {
    todo!()
}
