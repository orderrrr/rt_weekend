use itertools::Itertools;
use rayon::{prelude::*, result};
use rt_impl::{
    hittable::{HittableList, Sphere},
    render_px,
};
use std::{
    fs::File,
    io::BufWriter,
};

use spirv_std::glam::{uvec2, UVec2, Vec3};

pub fn render_cpu(wh: UVec2) {
    println!("Rendering on CPU with width, height: {}, {}", wh.x, wh.y);

    let file = File::create("output.png").unwrap();
    let w = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, wh.x, wh.y); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let c = rt_impl::ShaderConstants {
        width: wh.x,
        height: wh.y,
        aa_stages: 100,
    };

    let world = HittableList {
        list: vec![
            Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)),
            Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)),
        ],
    };

    let data: Vec<u8> = (0..wh.y)
        .into_iter()
        .cartesian_product(0..wh.x)
        .into_iter()
        .collect::<Vec<(u32, u32)>>()
        .par_iter()
        .map(|(h, w)| {
            let result = render_px(&c, &world, uvec2(*w, *h));
            vec![
                (result.x * 255.999) as u8,
                (result.y * 255.999) as u8,
                (result.z * 255.999) as u8,
            ]
        })
        .flatten()
        .collect();

    writer.write_image_data(&data).unwrap();
}
