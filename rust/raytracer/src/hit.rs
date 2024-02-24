use super::vec::{Vec3, Point3};
use super::ray::Ray;

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool
}

impl HitRecord {
    // Note: This is dodgy, I'm changing their code, but I really don't like
    // making the HitRecord with dummy values only to overwrite them.
    // And this way, I can have correctly initialized HitRecords without
    // necessarily making them mutable.
    pub fn with_normal_against_ray(p: Point3, t: f64, r: &Ray, outward_normal: Vec3) -> HitRecord {
        let front_face = r.direction().dot(outward_normal) > 0.0;
        HitRecord {
            p,
            normal: if front_face  { outward_normal } else { -1.0 * outward_normal },
            t,
            front_face
        }
    }
}



pub trait Hit {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}