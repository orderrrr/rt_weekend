use std::mem;

use spirv_std::glam::{UVec2, Vec2};

const IEEE_MANTISSA: u32 = 0x007FFFFF; // binary32 mantissa bitmask
const IEEE_ONE: u32 = 0x3F800000; // 1.0 in IEEE binary32

pub fn degrees_to_radians(degrees: f32) -> f32 {
    return degrees * std::f32::consts::PI / 180.0;
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
                let x = rand_f32(Vec2::new(x as f32 / max as f32, y as f32 / max as f32));
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
