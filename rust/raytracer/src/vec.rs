use std::ops::{Index, IndexMut, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Range};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
pub struct Vec3 {
    e: [f64; 3]
}

pub type Point3 = Vec3;
pub type Color = Vec3;

use std::fmt;
use std::fmt::Display;

impl Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}. {}. {})", self[0], self[1], self[2])
    }
}

impl Vec3 {
    pub fn new(e0: f64, e1: f64, e2: f64) -> Vec3 {
        Vec3 {
            e: [e0, e1, e2]
        }
    }

    pub fn random(rng: &mut impl Rng, r: Range<f64>) -> Vec3 {
        Vec3 {
            e: [rng.gen_range(r.clone()), rng.gen_range(r.clone()), rng.gen_range(r.clone())]
        }
    }

    pub fn random_in_unit_sphere(rng: &mut impl Rng) -> Vec3 {
        loop {
            let v = Vec3::random(rng, -1.0..1.0);
            if v.length() < 1.0 { // The vector is within the unit sphere.
                return v;
            }
        }
    }

    pub fn random_in_hemisphere(rng: &mut impl Rng, initial_direction: Vec3) -> Vec3 {
        let in_unit_sphere = Vec3::random_in_unit_sphere(rng);

        if in_unit_sphere.dot(initial_direction) > 0.0 {
            in_unit_sphere
        } else {
            -1.0 * in_unit_sphere
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &f64 {
        // I think this is inferring the lifetime of this reference to be the
        // same as the lifetime of the reference to self.
        &self.e[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        &mut self.e[index]
    }
}

//
// Memberwise addition + subtraction for Vec3:
//

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            e: [self[0] + other[0], self[1] + other[1], self[2] + other[2]]
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        *self = Vec3 {
            e: [self[0] + other[0], self[1] + other[1], self[2] + other[2]]
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            e: [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        *self = Vec3 {
            e: [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
        }
    }
}

//
// Scalar Multiplication for Vec3 (commutative, scroll down for f64-on-the left
// cases):
//

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: f64) -> Vec3 {
        Vec3 {
            e: [self[0] * other, self[1] * other, self[2] * other]
        }
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, other: f64) {
        *self = Vec3 {
            e: [self[0] * other, self[1] * other, self[2] * other]
        }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            e: [self * other[0], self * other[1], self * other[2]]
        }
    }
}

// This seems to be required for the multiplication of attenuation and Color to
// work. It's not shown in the rust tutorial (or, as far as I can tell, the C++
// tutorial), but hey.
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            e: [self[0] * other[0], self[1] * other[1], self[2] * other[2]]
        }
    }
}

//
// Division by scalars for Vec3:
//

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, other: f64) -> Vec3 {
        Vec3 {
            e: [self[0] / other, self[1] / other, self[2] / other]
        }
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, other: f64) {
        *self = Vec3 {
            e: [self[0] / other, self[1] / other, self[2] / other]
        }
    }
}

//
// Utility functions:
//
impl Vec3 {

    // Poor man's swizzles (not in the CPP version, as far as I can tell):

    pub fn x(self) -> f64 {
        self[0]
    }

    pub fn y(self) -> f64 {
        self[1]
    }

    pub fn z(self) -> f64 {
        self[2]
    }

    pub fn dot(self, other: Vec3) -> f64 {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt() // interesting, looks like these can be used with method-call syntax too.
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            e: [
                self[1] * other[2] - self[2] * other[1],
                self[2] * other[0] - self[0] * other[2],
                self[0] * other[1] - self[1] * other[0],
            ]
        }
    }

    pub fn normalized(self) -> Vec3 {
        self / self.length()
    }

    pub fn near_zero(self) -> bool {
        const EPS: f64 = 1.0e-8;
        self[0].abs() < EPS && self[1].abs() < EPS && self[2].abs() < EPS
    }

    pub fn reflect(self, n: Vec3) -> Vec3 {
        self - 2.0 * self.dot(n) * n
    }
}

// Color specific utility functions:

impl Vec3 {
    pub fn format_color(self, samples_per_pixel: u64) -> String {
        format!(
            "{} {} {}",
            (256.0 * (self[0] / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999)) as u64,
            (256.0 * (self[1] / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999)) as u64,
            (256.0 * (self[2] / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999)) as u64,
        )
    }
}