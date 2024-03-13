use rand::Rng;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};

use super::hit::HitRecord;
use super::ray::Ray;
use super::vec::{Vec3, Color};

#[typetag::serde(tag = "type")]
pub trait Scatter {
    fn scatter(&self, rng: &mut ChaCha12Rng, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

#[derive(Serialize, Deserialize)]
pub struct Lambertian {
    albedo: Color
}

impl Lambertian {
    #[allow(unused)]
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            albedo
        }
    }
}

#[typetag::serde]
impl Scatter for Lambertian {
    fn scatter(&self, rng: &mut ChaCha12Rng, _r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
            // I believe this return tuple should be thought of as "(attenuation, direction)".

        //
        // Some kinds of distributions (consider switching between these at various
        // stages):
        //

        // *Almost* right. cos^3(theta) instead of cos(theta).
        #[allow(dead_code)]
        fn initial_hack(rng: &mut impl Rng, rec: &HitRecord) -> Vec3 {
            rec.normal + Vec3::random_in_unit_sphere(rng)
        }

        // Correct. cos(theta)
        fn lambertian(rng: &mut impl Rng, rec: &HitRecord) -> Vec3 {
            rec.normal + Vec3::random_in_unit_sphere(rng).normalized()
        }

        // Naive.
        #[allow(dead_code)]
        fn simple_hemisphere(rng: &mut impl Rng, rec: &HitRecord) -> Vec3 {
            Vec3::random_in_hemisphere(rng, rec.normal)
        }

        let mut scatter_direction = lambertian(rng, rec);

        if scatter_direction.near_zero() {
            // TODO: This being Rust, there's probably a more functional way to
            // do this. But I'll leave it like this for now.
            scatter_direction = rec.normal;
        }
        let scattered = Ray::new(rec.p, scatter_direction);

        Some((self.albedo, scattered))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Metal {
    albedo: Color,
    fuzz: f64
}

impl Metal {
    #[allow(unused)]
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal {
            albedo,
            fuzz
        }
    }
}

#[typetag::serde]
impl Scatter for Metal {
    fn scatter(&self, rng: &mut ChaCha12Rng, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflection_direction = r_in.direction().reflect(rec.normal).normalized();
            // It seems like we don't really need to renormalize this, even
            // though we aren't keeping it normal. What gives?
        let scattered = Ray::new(rec.p, reflection_direction + self.fuzz * Vec3::random_in_unit_sphere(rng));
        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            // Now, since we're adding a random perturbation to the direction of
            // our reflected ray, we need to handle this case because we might have
            // made the ray go into the sphere.
            None
        }
    }
}