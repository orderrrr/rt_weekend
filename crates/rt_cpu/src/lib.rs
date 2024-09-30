use itertools::Itertools;
use rayon::prelude::*;
use rt_impl::{depth::render_depth_pass, describe_scene, render_pass_one, ShaderConstants};
use std::{fs::File, io::BufWriter};

use spirv_std::glam::{uvec2, UVec2, Vec4};

pub fn render_cpu(wh: UVec2) {
    println!("Rendering on CPU with width, height: {}, {}", wh.x, wh.y);

    let file = File::create("output.png").unwrap();
    let w = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, wh.x, wh.y); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let c = ShaderConstants {
        width: wh.x,
        height: wh.y,
        aa_stages: 150,
        bounce_limit: 100,
        focus_point: 78.0,
    };

    let world = describe_scene();

    let iter: Vec<(u32, u32)> = (0..wh.y)
        .into_iter()
        .cartesian_product(0..wh.x)
        .into_iter()
        .collect::<Vec<(u32, u32)>>();

    let pass_one: Vec<Vec4> = iter
        .par_iter()
        .map(|(h, w)| render_pass_one(&c, &world, uvec2(*w, *h)))
        .collect();

    let depth_pass: Vec<Vec4> = iter
        .par_iter()
        .map(|(h, w)| render_depth_pass(&c, &world, uvec2(*w, *h), &pass_one))
        .collect();

    let data: Vec<u8> = depth_pass
        .par_iter()
        .map(|d| {
            vec![
                (d.x * 255.999) as u8,
                (d.y * 255.999) as u8,
                (d.z * 255.999) as u8,
            ]
        })
        .flatten()
        .collect();

    writer.write_image_data(&data).unwrap();
}
