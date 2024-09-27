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
    Dialetric(DialetricMaterial),
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
            MaterialE::Dialetric(m) => m.scatter(r_in, hit),
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
    pub fuzz: f32,
    pub albedo: Vec3,
}

impl MetalMaterial {
    pub fn new(albedo: Vec3, fuzz: f32) -> Self {
        Self { albedo, fuzz }
    }
}

impl Default for MetalMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.5),
            fuzz: 0.0,
        }
    }
}

impl Material for MetalMaterial {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> MatResult {
        let rfl = util::reflect(r_in.direction, hit.normal).normalize()
            + (self.fuzz * util::random_in_unit_sphere(r_in.seed * 1.029838));

        let ray = if rfl.dot(hit.normal) > 0.0 {
            Some(Ray::new(hit.position, rfl, hash22(r_in.seed * 1.0012032)))
        } else {
            None
        };

        MatResult {
            ray,
            attenuation: self.albedo,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DialetricMaterial {
    pub albedo: Vec3,
    pub refractive_index: f32,
}

impl DialetricMaterial {
    pub fn new(albedo: Vec3, refractive_index: f32) -> Self {
        Self {
            albedo,
            refractive_index,
        }
    }

    fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
        // Use Schlick's approximation for reflectance.
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        (r0 * r0) + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Default for DialetricMaterial {
    fn default() -> Self {
        Self {
            albedo: Vec3::splat(0.5),
            refractive_index: 1.5,
        }
    }
}

impl Material for DialetricMaterial {
    fn scatter(&self, r: &Ray, h: &Hit) -> MatResult {
        let ri = if h.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };

        let unit_direction = r.direction.normalize();
        let cos_theta = (-unit_direction).dot(h.normal).min(1.0);
        let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract
            || DialetricMaterial::reflectance(cos_theta, ri) > util::rand_f32(r.seed.x)
        {
            util::reflect(unit_direction, h.normal)
        } else {
            util::refract(unit_direction, h.normal, ri)
        };

        MatResult {
            ray: Some(Ray::new(h.position, direction, hash22(r.seed * 1.0012032))),
            attenuation: self.albedo,
        }
    }
}

pub struct MatResult {
    pub ray: Option<Ray>,
    pub attenuation: Vec3,
}
