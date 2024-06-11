use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::sphere::Sphere;

use super::hit::HitRecord;

use super::ray::Ray;

use super::hit::Hit;

use super::material::Material;

use super::vec::{Point3, Vec3};

#[derive(Serialize, Deserialize)]
pub struct Plane {
    any_point: Point3,
    normal: Vec3,
    mat: Arc<dyn Material>,
}

impl Plane {
    #[allow(unused)]
    pub fn new(any_point: Point3, normal: Vec3, mat: Arc<dyn Material>) -> Plane {
        Plane {
            any_point,
            normal,
            mat,
        }
    }
}

#[typetag::serde]
impl Hit for Plane {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denominator = Vec3::dot(self.normal, r.direction());
        if denominator == 0.0 {
            return None;
        }

        let t = Vec3::dot(self.normal, self.any_point - r.origin()) / denominator;

        if t < t_min || t > t_max {
            return None;
        }

        let p = r.at(t);

        let rec = HitRecord::with_normal_against_ray(p, t, r, self.normal, Arc::clone(&self.mat));

        Some(rec)
    }
    fn collides_with_sphere(&self, _: &Sphere) -> bool {
        // TODO: Implement this collision. *Should* only be used in generating the scene, so I *should* be fine, but still.
        false
    }
}
