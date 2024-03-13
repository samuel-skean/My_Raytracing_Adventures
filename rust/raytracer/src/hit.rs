use std::rc::Rc;

use super::material::Scatter;
use super::vec::{Vec3, Point3};
use super::ray::Ray;

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Rc<dyn Scatter>,
    pub t: f64,
    pub front_face: bool
}

impl HitRecord {
    // Note: This is dodgy, I'm changing their code, but I really don't like
    // making the HitRecord with dummy values only to overwrite them.
    // And this way, I can have correctly initialized HitRecords without
    // necessarily making them mutable.
    pub fn with_normal_against_ray(p: Point3, t: f64, r: &Ray, outward_normal: Vec3, mat: Rc<dyn Scatter>) -> HitRecord {
        let front_face = r.direction().dot(outward_normal) < 0.0;
        HitRecord {
            p,
            normal: if front_face { outward_normal } else { -1.0 * outward_normal },
            t,
            mat,
            front_face
        }
    }
}


#[typetag::serde(tag = "type")]
pub trait Hit {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

pub type World = Vec<Box<dyn Hit>>;
    // impl's and clean type aliases do reduce boilerplate a lot here
    // I kinda wish I could safely inherit from this type,
    // making this a new type but still with all those methods.

#[typetag::serde]
impl Hit for World {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut tmp_rec = None;

        let mut closest_so_far = t_max;

        for object in self {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                    // Using closest_so_far as t_max makes sure we only get hits that are
                    // closer than all the things this ray has hit so far.
                closest_so_far = rec.t;
                tmp_rec = Some(rec);
            }
        }

        tmp_rec
    }
}