use rand::Rng;
use rand_chacha::ChaCha12Rng;

use super::hit::HitRecord;
use super::ray::Ray;
use super::vec::{Vec3, Color};

pub trait Scatter {
    fn scatter(&self, rng: &mut ChaCha12Rng, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            albedo
        }
    }
}

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

pub struct Metal {
    albedo: Color
}

impl Metal {
    pub fn new(albedo: Color) -> Metal {
        Metal {
            albedo
        }
    }
}

impl Scatter for Metal {
    fn scatter(&self, rng: &mut ChaCha12Rng, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflection_direction = r_in.direction().reflect(rec.normal);
        let reflected = Ray::new(rec.p, reflection_direction);
        Some((self.albedo, reflected))
    }
}