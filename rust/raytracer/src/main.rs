mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;
mod material;

use std::{io::{stderr, Write}, rc::Rc};
use rand::{Rng, SeedableRng};

use rand_chacha::ChaCha12Rng;
use vec::{Vec3, Point3, Color};
use ray::Ray;
use hit::{Hit, World};
use sphere::Sphere;
use camera::Camera;

use material::{Scatter, Lambertian, Metal};

// Gets a color from each ray that forms a gradient when put together in the
// viewport.
// Because the ray is normalized first, there is a slight horizontal gradient
// from light blue on the left, through white, and to light blue on the right.
// Basically, the x stole from the y when it was pointing left and pointing
// right. This is why the image is pretty :).
fn ray_color(r: &Ray, world: &World, depth: u64, rng: &mut ChaCha12Rng) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(rng, r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1, rng)
        }
        else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u64 = 256;
const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
const SAMPLES_PER_PIXEL: u64 = 100;
const MAX_DEPTH: u64 = 5;

fn main() {

    // World
    let mut world = World::new();


    let mut rng = ChaCha12Rng::seed_from_u64(0);
    for _ in 0..200 {
        let rand_color = Color::new(rng.gen(), rng.gen(), rng.gen());
        let rand_mat: Rc<dyn Scatter> = if rng.gen_bool(0.6) {
            Rc::new(Metal::new(rand_color, rng.gen()))
        } else {
            Rc::new(Lambertian::new(rand_color))
        };
        let sphere = if rng.gen_bool(0.9) {
            Sphere::new(
                Point3::new(
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-0.5..1.0),
                    rng.gen_range(-2.0..-1.0)
                ),
                rng.gen_range(0.0..0.4),
                rand_mat
            )
        } else {
            Sphere::new(
                Point3::new(
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..-25.0)
                ),
                rng.gen_range(15.0..20.0),
                rand_mat
            )
        };
        world.push(Box::new(sphere));
    }

    // Camera
    let cam = Camera::new();

    // Header
    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {:4}", j + 1);
        stderr().flush().unwrap();

        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Vec3::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let random_u_component: f64 = rng.gen();
                let random_v_component: f64 = rng.gen();

                let u =
                    ((i as f64) + random_u_component) / ((IMAGE_WIDTH - 1) as f64);
                let v =
                    ((j as f64) + random_v_component) / ((IMAGE_HEIGHT - 1) as f64);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH, &mut rng);
            }

            print!("{} ", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}