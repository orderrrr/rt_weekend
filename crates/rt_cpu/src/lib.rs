use rt_impl::{
    hittable::{HittableList, Sphere},
    render_px,
};
use std::{fs::File, io::BufWriter};

use spirv_std::glam::{uvec2, UVec2, Vec3};

pub fn render(wh: UVec2) {
    println!("Rendering on CPU with width, height: {}, {}", wh.x, wh.y);

    let file = File::create("output.png").unwrap();
    let w = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, wh.x, wh.y); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let mut data = vec![0u8; (wh.x * wh.y * 3) as usize];

    let c = rt_impl::ShaderConstants {
        width: wh.x,
        height: wh.y,
    };

    let world = HittableList {
        list: vec![
            Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)),
            Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)),
        ],
    };

    for h in 0..wh.y {
        if h % 10 == 0 {
            println!("Rows Remaining: {}", wh.y - h);
        }

        for w in 0..wh.x {
            let result = render_px(&c, &world, uvec2(w, h));
            let idx = ((h * wh.x + w) * 3) as usize;
            data[idx + 0] = (result.x * 255.999) as u8;
            data[idx + 1] = (result.y * 255.999) as u8;
            data[idx + 2] = (result.z * 255.999) as u8;
        }
    }

    writer.write_image_data(&data).unwrap();
}
