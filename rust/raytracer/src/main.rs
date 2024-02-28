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

use material::{Lambertian, Metal};

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

    let mat_center = Rc::new(Lambertian::new(Color::new(0.7, 0.3, 0.3)));
    let mat_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_left = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.3));
    let mat_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center)));
        // a lil ball
    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground)));
        // the Earth!
    world.push(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left)));
        // left metal ball
    world.push(Box::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right)));
        // right metal ball

    // Camera
    let cam = Camera::new();

    // Header
    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    let mut rng = ChaCha12Rng::seed_from_u64(0);
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