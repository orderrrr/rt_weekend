use std::f32::INFINITY;

use spirv_std::glam::Vec3;

use crate::ray::Ray;

pub struct Hit {
    pub p: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
}

pub trait Hitable {
    fn hit(&self, r: &Ray, t: Interval) -> Option<Hit>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Hitable for Sphere {
    fn hit(&self, r: &Ray, t: Interval) -> Option<Hit> {
        let oc = self.center - r.origin;
        let a = r.direction.length_squared();
        let h = r.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = (h - discriminant.sqrt()) / a;
        let mut root = sqrtd.clamp(t.min, t.max);

        if !t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !t.surrounds(root) {
                return None;
            }
        }

        let p = r.origin + (r.direction * root);
        let outward_normal = ((p - self.center) / self.radius).normalize();

        let front_face = r.direction.dot(outward_normal) < 0.0;
        // invert normal if we are inside

        let n = 2.0 * f32::from(front_face) - 1.0;
        let normal = outward_normal * n;

        Some(Hit {
            p,
            normal,
            front_face,
            t: root,
        })
    }
}

pub struct HittableList {
    pub list: Vec<Box<dyn Hitable>>,
}

impl Hitable for HittableList {
    fn hit(&self, r: &Ray, t: Interval) -> Option<Hit> {
        let mut closest = t.max;
        let mut hit: Option<Hit> = None;

        for h in self.list.iter() {
            match h.hit(r, Interval::new(t.min, closest)) {
                Some(r) => {
                    closest = r.t;
                    hit = Some(r);
                }
                None => {}
            }
        }

        hit
    }
}

pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub fn new(min: f32, max: f32) -> Self {
        Interval { min, max }
    }

    pub fn empty() -> Self {
        Interval {
            min: INFINITY,
            max: -INFINITY,
        }
    }

    pub fn universe() -> Self {
        Interval {
            min: -INFINITY,
            max: INFINITY,
        }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn contains(&self, x: f32) -> bool {
        min(self.min, self.max) <= x && x <= max(self.min, self.max)
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }
}

pub fn min(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
