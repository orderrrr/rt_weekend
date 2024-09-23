use spirv_std::glam;

use glam::Vec3;

pub trait Saturate {
    fn saturate(self) -> Self;
}

impl Saturate for Vec3 {
    fn saturate(self) -> Self {
        let mut v = self;
        v.x = v.x.clamp(0.0, 1.0);
        v.y = v.y.clamp(0.0, 1.0);
        v.z = v.z.clamp(0.0, 1.0);
        v
    }
}
