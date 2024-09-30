use spirv_std::glam::{uvec2, vec2, vec3, vec4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{hittable::HittableE, util::linear_to_gamma, ShaderConstants};

pub const MAX_BLUR_SIZE: f32 = 20.0;
pub const GOLDEN_ANGLE: f32 = 2.39996322972865332;
pub const UFAR: f32 = 10.0;
pub const RAD_SCALE: f32 = 0.5;

pub fn blur_size(depth: f32, focus_point: f32, focus_scale: f32) -> f32 {
    let coc = ((1.0 / focus_point - 1.0 / depth) * focus_scale).clamp(-1.0, 1.0);
    coc.abs() * MAX_BLUR_SIZE
}

pub fn uv_to_id(sc: &ShaderConstants, uv: Vec2) -> usize {
    let v2 = (uv * uvec2(sc.width, sc.height).as_vec2())
        .round()
        .as_uvec2();

    wrap_id(sc, (v2.y * sc.width + v2.x) as usize)
}

pub fn wrap_id(sc: &ShaderConstants, id: usize) -> usize {
    (id + (sc.width * sc.height) as usize) % (sc.width * sc.height) as usize
}

pub fn depth_of_field(
    sc: &ShaderConstants,
    uv: Vec2,
    focus_point: f32,
    focus_scale: f32,
    tex: &Vec<Vec4>,
) -> Vec3 {
    let i = tex[uv_to_id(sc, uv)];

    let center_depth = i.w * UFAR;
    let center_size = blur_size(center_depth, focus_point, focus_scale);
    let mut color = i.xyz();
    let mut tot = 1.0;

    let tex_size = 1.0 / uvec2(sc.width, sc.height).as_vec2();

    let mut radius = RAD_SCALE;
    let mut ang = 0.0;
    while radius < MAX_BLUR_SIZE {
        ang += GOLDEN_ANGLE;
        let tc = uv + vec2(ang.cos(), ang.sin()) * tex_size * radius;

        let sample_input = tex[uv_to_id(sc, tc)];

        let sample_color = sample_input.xyz();
        let sample_depth = sample_input.w * UFAR;
        let mut sample_size = blur_size(sample_depth, focus_point, focus_scale);

        if sample_depth > center_depth {
            sample_size = sample_size.clamp(0.0, center_size * 2.0);
        }

        let m = smoothstep(radius - 0.5, radius + 0.5, sample_size);
        color += mix(color / tot, sample_color, m);
        tot += 1.0;
        radius += RAD_SCALE / radius;
    }

    color / tot
}

// todo is this correct?
fn smoothstep(edge_0: f32, edge_1: f32, x: f32) -> f32 {
    let t = ((x - edge_0) / (edge_1 - edge_0)).clamp(0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn mix(a: Vec3, b: Vec3, x: f32) -> Vec3 {
    (a * (1.0 - x)) + (b * x)
}

pub fn render_depth_pass(
    sc: &ShaderConstants,
    _world: &HittableE,
    idx: UVec2,
    pass_one: &Vec<Vec4>,
) -> Vec4 {
    let uv = idx.as_vec2() / uvec2(sc.width, sc.height).as_vec2();

    // float focusPoint = 78.0;
    // float focusScale = (0.5 + sin(iTime) * 0.5) * 50.0;
    let focus_point = 30.;
    let focus_scale = 1.0 * 50.0;

    let mut color = depth_of_field(sc, uv, focus_point, focus_scale, pass_one);
    // tone mapping
    // color = vec3(1.7, 1.8, 1.9) * color / (1.0 + color);

    // color = color.powf(1.0 / 1.8);
    color = linear_to_gamma(color);

    vec4(color.x, color.y, color.z, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rayon::prelude::*;

    #[test]
    fn test_uv_to_id() {
        let h = 400;
        let w = 800;

        let sc = ShaderConstants {
            width: w,
            height: h,
            aa_stages: 150,
            bounce_limit: 100,
            focus_point: 1.0,
        };

        let correct: Vec<(u32, u32)> = (0..w)
            .into_iter()
            .cartesian_product(0..h)
            .into_iter()
            .collect::<Vec<(u32, u32)>>();

        let result: Vec<(u32, u32)> = correct
            .clone()
            .into_par_iter()
            .map(|(j, i)| {
                let uv = uvec2(i, j).as_vec2() / uvec2(sc.width, sc.height).as_vec2();

                let v2 = (uv * uvec2(sc.width, sc.height).as_vec2())
                    .round()
                    .as_uvec2();

                assert_eq!(v2.x, i);
                assert_eq!(v2.y, j);

                (v2.y, v2.x)
            })
            .collect();

        // check if result is the same
        assert_eq!(result, correct);
    }
}
