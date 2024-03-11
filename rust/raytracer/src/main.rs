mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;

use std::io::{stderr, Write};
use clap::{error::ErrorKind, CommandFactory as _, Parser};
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

#[derive(Parser, Debug)]
struct Cli {
    // NOTE: I am including a description of the defaults in the doc comments
    // like the default messages that clap would generate if I were specifying
    // them as having default values for some of these values.
    // This is a bit ugly, but necessary if I want to distinguish between the
    // arguments being specified and the arguments not being specified - I can't
    // just have clap fill them in.

    /// Aspect ratio [default: 16.0 9.0]
    #[arg(short = 'r', long, value_delimiter = ' ', num_args = 2)]
        // Why doesn't this seem to work when I set value_delimiter to something
        // other than ' '?
        // Why can't I have this somehow parse into a tuple?
        // Thankfully, this seems to require the code have the two components of
        // the aspect ratio be consecutive, but why is unintuitive - it seems
        // from the docs like maybe $ cmd -r 16 -s 200 -r 10 should work to
        // supply [16, 9] as the value of aspect_ratio, but nope.
    aspect_ratio: Option<Vec<f64>>,
    /// Image width [default: 256]
    #[arg(short = 'w', long)]
    image_width: Option<u64>,
    /// Image height [default: 144]
    #[arg(short = 'H', long)]
    image_height: Option<u64>,
    /// Samples per pixel
    #[arg(short, long, default_value_t = 100)]
    samples_per_pixel: u64,
    /// Max bounce depth
    #[arg(short = 'b', long = "bounces", default_value_t = 5)]
    max_depth: u64,
}

struct Resolution {
    width: u64,
    height: u64,
}

fn get_aspect_ratio_and_resolution(aspect_ratio: Option<Vec<f64>>, width: Option<u64>, height: Option<u64>) -> (f64, Resolution) {
    const DEFAULT_HEIGHT: u64 = 144;
    const DEFAULT_ASPECT_RATIO: f64 = 16.0 / 9.0;

    let aspect_ratio_was_specified = aspect_ratio.is_some(); // This is kinda icky.
    let aspect_ratio = aspect_ratio
        .and_then(|ratio_vec: Vec<f64>| Some(ratio_vec[0] / ratio_vec[1]))
        .unwrap_or(DEFAULT_ASPECT_RATIO);

    let resolution = match (width, height) {
        (Some(_), Some(_)) if aspect_ratio_was_specified => {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Can only specify one resolution dimension along with an aspect ratio."
            )
            .exit();
        }
        (Some(width), Some(height)) => {
            return (width as f64 / height as f64, Resolution { width, height });
            // This early return in a match seems very icky. But it also seems
            // like the best way to do it!
            // I kinda wish I had, I dunno, a switch statement.
        }
        (Some(width), None) => Resolution {
            width,
            height: (width as f64 / aspect_ratio) as u64,
        },
        (None, Some(height)) => Resolution {
            width: (aspect_ratio * height as f64) as u64,
            height,
        },
        (None, None) => Resolution {
            width: (aspect_ratio * DEFAULT_HEIGHT as f64) as u64,
            height: DEFAULT_HEIGHT,
        },
    };

    (aspect_ratio, resolution)
}

fn main() {

    let args = Cli::parse();

    let (aspect_ratio, res) = get_aspect_ratio_and_resolution(args.aspect_ratio, args.image_width, args.image_height);

    // World
    let mut world = World::new();
    world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
        // a lil ball
    world.push(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));
        // the Earth!

    // Camera
    let cam = Camera::new(f64::from(aspect_ratio));

    // Header
    println!("P3");
    println!("{} {}", res.width, res.height);
    println!("255");

    let mut rng = ChaCha12Rng::seed_from_u64(0);
    for j in (0..res.height).rev() {
        eprint!("\rScanlines remaining: {:4}", j + 1);
        stderr().flush().unwrap();

        for i in 0..res.width {
            let mut pixel_color = Vec3::new(0.0, 0.0, 0.0);
            for _ in 0..args.samples_per_pixel {
                let random_u_component: f64 = rng.gen();
                let random_v_component: f64 = rng.gen();

                let u =
                    ((i as f64) + random_u_component) / ((res.width - 1) as f64);
                let v =
                    ((j as f64) + random_v_component) / ((res.height - 1) as f64);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, args.max_depth, &mut rng);
            }

            print!("{} ", pixel_color.format_color(args.samples_per_pixel));
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}