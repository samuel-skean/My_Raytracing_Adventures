mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;

use std::io::{stderr, Write};
use clap::Parser;
use rand::{Rng, SeedableRng};

use rand_chacha::ChaCha12Rng;
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
fn ray_color(r: &Ray, world: &World, depth: u64, rng: &mut impl Rng) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        let target = rec.p + rec.normal + Vec3::random_in_unit_sphere(rng).normalized();
        let r = Ray::new(rec.p, target - rec.p);
        0.5 * ray_color(&r, world, depth - 1, rng)
            // Sphere reflects half the light it gets.
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}


// const ASPECT_RATIO: f64 = 16.0 / 9.0;
// const IMAGE_WIDTH: u64 = 256;
// const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
// const SAMPLES_PER_PIXEL: u64 = 100;
// const MAX_DEPTH: u64 = 5;

#[derive(Parser, Debug)]
struct Args {
    /// Aspect Ratio
    #[arg(short = 'r', long, value_delimiter = ' ', num_args = 2, default_value = "16.0 9.0")]
        // Why doesn't this seem to work when I set value_delimiter to something
        // other than ' '?
        // Also, why is value_delimiter necessary to set to get default_value to
        // work properly, and how hard would it be to switch to default_value_t?
        // Why can't I have this somehow parse into a tuple?
        // Thankfully, this seems to require the code have the two components of
        // the aspect ratio be consecutive, but why is unintuitive - it seems
        // from the docs like maybe $ cmd -r 16 -s 200 -r 10 should work to
        // supply [16, 9] as the value of aspect_ratio, but nope.
        // Also, definitely consider StructOpt, if nothing else for the
        // structopt-yaml crate, which seems to do exactly what I want when it
        // comes to providing some options on the command line that override
        // options in a config file that override options inherent to the program.
    aspect_ratio: Vec<f64>,
    /// Image Height
    #[arg(short = 'H', long, default_value_t = 144)]
    image_height: u64,
    /// Samples Per Pixel
    #[arg(short, long, default_value_t = 100)]
    samples_per_pixel: u64,
    /// Max Bounce Depth
    #[arg(short = 'b', long = "bounces", default_value_t = 5)]
    max_depth: u64,
}

fn main() {

    let args = Args::parse();

    let aspect_ratio = args.aspect_ratio[0] / args.aspect_ratio[1];
    let image_width = (aspect_ratio * (args.image_height as f64)) as u64;

    // World
    let mut world = World::new();
    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
        // a lil ball
    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));
        // the Earth!

    // Camera
    let cam = Camera::new(aspect_ratio);

    // Header
    println!("P3");
    println!("{} {}", image_width, args.image_height);
    println!("255");

    let mut rng = ChaCha12Rng::seed_from_u64(0);
    for j in (0..args.image_height).rev() {
        eprint!("\rScanlines remaining: {:4}", j + 1);
        stderr().flush().unwrap();

        for i in 0..image_width {
            let mut pixel_color = Vec3::new(0.0, 0.0, 0.0);
            for _ in 0..args.samples_per_pixel {
                let random_u_component: f64 = rng.gen();
                let random_v_component: f64 = rng.gen();

                let u =
                    ((i as f64) + random_u_component) / ((image_width - 1) as f64);
                let v =
                    ((j as f64) + random_v_component) / ((args.image_height - 1) as f64);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, args.max_depth, &mut rng);
            }

            print!("{} ", pixel_color.format_color(args.samples_per_pixel));
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}