use spirv_std::glam::Vec3;

use crate::{
    hittable::Hit,
    ray::Ray,
    util::{self, hash22},
};

pub trait Material {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> MatResult;
}

#[derive(Copy, Clone)]
pub enum MaterialE {
    Default(DefaultMaterial),
    Lambertian(LambertianMaterial),
    Metal(MetalMaterial),
}

impl Default for MaterialE {
    fn default() -> Self {
        Self::Default(DefaultMaterial::default())
    }
}

impl Material for MaterialE {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> MatResult {
        match self {
            MaterialE::Default(m) => m.scatter(r_in, hit),
            MaterialE::Lambertian(m) => m.scatter(r_in, hit),
            MaterialE::Metal(m) => m.scatter(r_in, hit),
        }
    }
}

#[derive(Copy, Clone)]
pub struct DefaultMaterial {
    pub albedo: Vec3,
}

impl Default for DefaultMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.5),
        }
    }
}

impl Material for DefaultMaterial {
    fn scatter(&self, _r_in: &Ray, _hit: &Hit) -> MatResult {
        MatResult {
            ray: None,
            attenuation: self.albedo,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LambertianMaterial {
    pub albedo: Vec3,
}

impl LambertianMaterial {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Default for LambertianMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.5),
        }
    }
}

impl Material for LambertianMaterial {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> MatResult {
        // let dir = hit.normal + util::random_in_unit_sphere(r_in.seed);
        let dir = util::random_on_hemisphere(hit.normal, r_in.seed);
        let ray = Ray::new(hit.position, dir, hash22(r_in.seed * 1.0012032));

        MatResult {
            ray: Some(ray),
            attenuation: self.albedo,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MetalMaterial {
    pub albedo: Vec3,
}

impl MetalMaterial {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Default for MetalMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.5),
        }
    }
}

impl Material for MetalMaterial {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> MatResult {
        let reflected = util::reflect(r_in.direction, hit.normal);
        let scattered = Ray::new(hit.position, reflected, hash22(r_in.seed * 1.0012032));
        MatResult {
            ray: Some(scattered),
            attenuation: self.albedo,
        }
    }
}

pub struct MatResult {
    pub ray: Option<Ray>,
    pub attenuation: Vec3,
}
