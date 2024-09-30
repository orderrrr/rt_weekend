use std::{f32::consts::PI, mem};

use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles};

const IEEE_MANTISSA: u32 = 0x007FFFFF; // binary32 mantissa bitmask
const IEEE_ONE: u32 = 0x3F800000; // 1.0 in IEEE binary32

pub fn degrees_to_radians(degrees: f32) -> f32 {
    return degrees * std::f32::consts::PI / 180.0;
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

pub fn hash22(p: Vec2) -> Vec2 {
    let mut p3 = (p.xyx() * vec3(0.1031, 0.1030, 0.0973)).fract();
    p3 += p3.dot(p3.yzx() + vec3(33.33, 33.33, 33.33));
    ((p3.xx() + p3.yz()) * p3.zy()).fract()
}

// vec3 hash32(vec2 p) {
//     vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
//     p3 += dot(p3, p3.yxz+33.33);
//     return fract((p3.xxy+p3.yzz)*p3.zyx);
// }
pub fn hash32(p: Vec2) -> Vec3 {
    let mut p3 = (p.xyx() * vec3(0.1031, 0.1030, 0.0973)).fract();
    p3 += p3.dot(p3.yxz() + Vec3::splat(33.33));
    ((p3.xxy() + p3.yzz()) * p3.zyx()).fract()
}

// vec3 randomInUnitSphere(vec2 p) {
//     vec3 rand = hash32(p);
//     float phi = 2.0 * PI * rand.x;
//     float cosTheta = 2.0 * rand.y - 1.0;
//     float u = rand.z;
//
//     float theta = acos(cosTheta);
//     float r = pow(u, 1.0 / 3.0);
//
//     float x = r * sin(theta) * cos(phi);
//     float y = r * sin(theta) * sin(phi);
//     float z = r * cos(theta);
//
//     return vec3(x, y, z);
// }
pub fn random_in_unit_sphere(p: Vec2) -> Vec3 {
    let rand = vec3(rand_f32(p.x), rand_f32(p.y), rand_f32(p.x));
    let phi = 2.0 * PI * rand.x;
    let cos_theta = 2.0 * rand.y - 1.0;
    let u = rand.z;

    let theta = cos_theta.acos();
    let r = u.powf(1.0 / 3.0);

    let x = r * theta.sin() * phi.cos();
    let y = r * theta.sin() * phi.sin();
    let z = r * theta.cos();

    Vec3::new(x, y, z)
}

pub fn random_on_hemisphere(normal: Vec3, seed: Vec2) -> Vec3 {
    let rd = random_in_unit_sphere(seed); // random vector from 0.0, 1.0
    let res = rd + normal;

    if res.abs() == Vec3::splat(0.0) {
        normal
    } else {
        res.normalize()
    }
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

// inline vec3 refract(const vec3& uv, const vec3& n, double etai_over_etat) {
//     auto cos_theta = std::fmin(dot(-uv, n), 1.0);
//     vec3 r_out_perp =  etai_over_etat * (uv + cos_theta*n);
//     vec3 r_out_parallel = -std::sqrt(std::fabs(1.0 - r_out_perp.length_squared())) * n;
//     return r_out_perp + r_out_parallel;
// }
pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = ((-uv).dot(n)).min(1.0);
    let r_out_perp = etai_over_etat * (uv + (cos_theta * n));
    let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}

// assumes none of the vectors length are zero
pub fn cosine_similarity(a: Vec3, b: Vec3) -> f32 {
    let dp = a.dot(b);
    let am = a.length();
    let bm = b.length();

    dp / (am * bm)
}

// Since we plan on running this in the gpu we cannot use any standard rust random libs.
// We will be using Bob Jenkins' smallprng for this.
fn hash(x: u32) -> u32 {
    let mut x = x;
    x += x << 10;
    x ^= x >> 6;
    x += x << 3;
    x ^= x >> 11;
    x += x << 15;
    x
}

pub fn float_to_u32(x: f32) -> u32 {
    unsafe { mem::transmute(x) }
}

pub fn vec2_to_u32(x: Vec2) -> UVec2 {
    UVec2::new(float_to_u32(x.x), float_to_u32(x.y))
}

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
pub fn rand_vec2(x: Vec2) -> f32 {
    let h = vec2_to_u32(x);
    let m = (hash(h.x ^ hash(h.y)) & IEEE_MANTISSA) | IEEE_ONE;

    let f: f32 = unsafe { mem::transmute(m) };
    return f - 1.0; // Range [0:1]
}

pub fn rand_f32(x: f32) -> f32 {
    let h = float_to_u32(x);
    let m = (hash(h) & IEEE_MANTISSA) | IEEE_ONE;

    let f: f32 = unsafe { mem::transmute(m) };
    return f - 1.0; // Range [0:1]
}

// we need to provide an input vector to rand_f32 so that we can use the same random seed
pub fn rand_vec3(x: Vec3) -> Vec3 {
    vec3(rand_f32(x.x), rand_f32(x.y), rand_f32(x.z))
}

pub fn disk_point(radius: f32, seed: Vec2) -> Vec2 {
    let (x1, x2) = (rand_f32(seed.x), rand_f32(seed.y));
    let p = radius * (1.0 - x1).sqrt();
    let theta = x2 * 2.0 * PI;
    vec2(p * theta.cos(), p * theta.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    #[test]
    pub fn test_rand_f32() {
        let mut i: u64 = 0;
        let max = 10_000;

        let mut max_results = 0.0;
        let mut min_results = 1.0;

        (0..max)
            .into_iter()
            .cartesian_product(0..max)
            .for_each(|(x, y)| {
                let x = rand_vec2(Vec2::new(x as f32 / max as f32, y as f32 / max as f32));
                i += 1;
                assert!(x >= 0.0);
                assert!(x <= 1.0);

                match x {
                    x if x > max_results => max_results = x,
                    x if x < min_results => min_results = x,
                    _ => {}
                }
            });

        println!("max: {}, min: {}", max_results, min_results);
        assert_eq!(max_results, 0.999_999_9);
        assert_eq!(min_results, 0.0);
        assert_eq!(i, max as u64 * max as u64);
    }
}
