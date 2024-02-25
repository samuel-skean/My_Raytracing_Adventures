mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;

use std::io::{stderr, Write};
use rand::Rng;

use vec::{Vec3, Point3, Color};
use ray::Ray;
use hit::{Hit, World};
use sphere::Sphere;
use camera::Camera;

// Gets a color from each ray that forms a gradient when put together in the
// viewport.
// Because the ray is normalized first, there is a slight horizontal gradient
// from light blue on the left, through white, and to light blue on the right.
// Basically, the x stole from the y when it was pointing left and pointing
// right. This is why the image is pretty :).
fn ray_color(r: &Ray, world: &World) -> Color {
    if let Some(rec) = world.hit(r, 0.0, f64::INFINITY) {
        0.5 * (rec.normal + Color::new(1.0,1.0, 1.0))
            // Mapping the components of the normal, which have the range -1.0
            // to 1.0 as normalized vectors, to the range 0 to 1.0
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {

    // Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u64 = 256;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 100;

    // World
    let mut world = World::new();
    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
        // a lil ball
    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));
        // the Earth!

    // Camera
    let cam = Camera::new();

    // Header
    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {:3}", j + 1);
        stderr().flush().unwrap();

        let mut rng = rand::thread_rng();
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
                pixel_color += ray_color(&r, &world);
            }

            print!("{} ", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}