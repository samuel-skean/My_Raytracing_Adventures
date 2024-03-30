use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::material::Material;
use super::vec::Point3;
use super::ray::Ray;
use super::hit::{Hit, HitRecord};

#[derive(Serialize, Deserialize)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Rc<dyn Material>
}

impl Sphere {
    #[allow(unused)]
    pub fn new(center: Point3, radius: f64, mat: Rc<dyn Material>) -> Sphere {
        Sphere {
            center,
            radius,
            mat
        }
    }
}

#[typetag::serde]
impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center; // A - C
        let a = r.direction().length().powi(2); // b . b
        let half_b = oc.dot(r.direction()); // b * (A - C)
        let c = oc.length().powi(2) - self.radius * self.radius; // (A - C) . (A - C) * r^2
        let quarter_discriminant = half_b * half_b - a * c;

        if quarter_discriminant < 0.0  { // there are no roots
            return None;
        }

        let half_sqrt_d = quarter_discriminant.sqrt();
        let mut root = (-half_b - half_sqrt_d) / a;
            // the smaller of the two roots (smaller t, so 'closer' to the
            // camera - assuming nothing's behind us)
        if root < t_min || root > t_max {
            // That root wasn't within the allowed range, try the other one.
            root = (-half_b + half_sqrt_d) / a;

            if root < t_min || root > t_max {
                // Neither root was in the allowed range.
                return None;
            }
        }
        let p = r.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let rec = HitRecord::with_normal_against_ray(p, root, r, outward_normal, self.mat.clone());

        Some(rec)
    }

    fn collides_with_sphere(&self, other: &Sphere) -> bool {
        (self.center - other.center).length() < (self.radius + other.radius)
    }
}