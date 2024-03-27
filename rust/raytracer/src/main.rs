mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;
mod material;

use std::{fs::File, io::{self, stderr, BufReader, BufWriter, Write}};
use clap_serde_derive::{clap::{self, error::ErrorKind, CommandFactory as _, Parser}, ClapSerde};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use rand::{rngs::ThreadRng, Rng};

use vec::{Vec3, Color};
use ray::Ray;
use hit::{Hit, World};
use camera::Camera;

// Gets a color from each ray that forms a gradient when put together in the
// viewport.
// Because the ray is normalized first, there is a slight horizontal gradient
// from light blue on the left, through white, and to light blue on the right.
// Basically, the x stole from the y when it was pointing left and pointing
// right. This is why the image is pretty :).
fn ray_color(r: &Ray, world: &World, depth: u64, rng: &mut ThreadRng) -> Color {
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

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    quiet: bool,
    #[command(flatten)]
    config: <Config as ClapSerde>::Opt,
}

#[derive(Debug, Serialize, Deserialize, ClapSerde)]
struct Config {
    // NOTE: I am faking all of the default arguments now, since I've
    // implemented my own logic for how they work. It's a shame that everything
    // has to appear in multiple places now. It seems like there's "a way" to
    // get this fixed up at compile time but it involves hella macros.
    // (https://stackoverflow.com/questions/72588743/can-you-use-a-const-value-in-docs-in-rust)

    /// Aspect ratio [default: 1.77 (same as $[16.0 / 9.0] in bash)]
    #[arg(short = 'r', long)]
    aspect_ratio: Option<f64>,
    /// Image width [default: 256]
    #[arg(short = 'W', long)]
    image_width: Option<u64>,
    /// Image height [default: 144]
    #[arg(short = 'H', long)]
    image_height: Option<u64>,
    /// Samples per pixel [default: 100]
    #[arg(short, long)]
    samples_per_pixel: u64,
    /// Max bounce depth [default: 5]
    #[arg(short = 'b', long = "bounces")]
    max_depth: u64,
    /// Path of the file to output to. If not specified, defaults to standard output.
    #[arg(short, long)]
    output_path: Option<std::path::PathBuf>,
    /// Random seed to use throughout the program, mostly for ray bounces.
    #[arg(short = 'R', long)]
    random_seed: u64,
    /// Only required if no config is specified.
    #[arg(required_unless_present("config_path"))]
    world_path: Option<std::path::PathBuf>,
}

struct Resolution {
    width: u64,
    height: u64,
}

fn get_aspect_ratio_and_resolution(aspect_ratio: Option<f64>, width: Option<u64>, height: Option<u64>) -> (f64, Resolution) {
    const DEFAULT_HEIGHT: u64 = 144;
    const DEFAULT_ASPECT_RATIO: f64 = 16.0 / 9.0;

    // Still a bit icky, but I could keep bikeshedding for days. I wish I had
    // something like a switch statement!

    let calculated_aspect_ratio = aspect_ratio.unwrap_or(DEFAULT_ASPECT_RATIO);

    match (aspect_ratio, width, height) {
        (Some(_), Some(_), Some(_)) => {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Can only specify one resolution dimension along with an aspect ratio."
            )
            .exit();
        }
        (None, Some(width), Some(height)) => {
            (width as f64 / height as f64, Resolution { width, height })
        }
        (_, Some(width), None) => (calculated_aspect_ratio, Resolution {
            width,
            height: (width as f64 / calculated_aspect_ratio) as u64,
        } ),
        (_, None, Some(height)) => (calculated_aspect_ratio, Resolution {
            width: (calculated_aspect_ratio * height as f64) as u64,
            height,
        }),
        (_, None, None) => (calculated_aspect_ratio, Resolution {
            width: (calculated_aspect_ratio * DEFAULT_HEIGHT as f64) as u64,
            height: DEFAULT_HEIGHT,
        }),
    }
}

fn main() -> io::Result<()> {
    let default_config = Config {
        aspect_ratio: None,
        image_width: None,
        image_height: None,
        samples_per_pixel: 100,
        max_depth: 5,
        output_path: None,
        random_seed: 0,
        world_path: None,
    };

    let args = Cli::parse();

    let config = match args.config_path {
        Some(path) => {
            let file = File::open(&path)?;
            default_config
                .merge(<Config as ClapSerde>::Opt::from(serde_json::from_reader::<
                    _,
                    <Config as ClapSerde>::Opt,
                >(
                    BufReader::new(file)
                )?))
                .merge(args.config)
        }
        None => {
            eprintln!(
                "No config file provided. Continuing with only the arguments and the defaults."
            );
            default_config.merge(<Config as ClapSerde>::Opt::from(args.config))
        }
    };

    let (aspect_ratio, res) = get_aspect_ratio_and_resolution(config.aspect_ratio, config.image_width, config.image_height);

    // Following this code: https://users.rust-lang.org/t/write-to-stdout-stderr-or-file/29739
    let mut output: Box<dyn io::Write> = match config.output_path {
        None => Box::new(io::stdout()),
        Some(ref output_path) => {
            let extension_error = || {
                panic!("The output path specified, {}, does not end in .ppm.", output_path.to_str()
                    .expect("The output path was not valid UTF-8."));
            };
            let Some(extension) = output_path.extension() else {
                extension_error()
            };
            if extension != "ppm" {
                extension_error()
            }
            Box::new(BufWriter::new(File::create(output_path)?))
        }
    };

    if !args.quiet {
        eprintln!("Using this configuration: {}", serde_json::to_string_pretty(&config)?);
        // TODO: This presents the configuration in a pretty way, but it doesn't
        // fill in anything that the program has computed from the input that
        // wasn't directly specified (that is, there could be a missing aspect
        // ratio, or dimension to the resolution - in fact, I think their must
        // be!). In an ideal world, the output here would be valid to feed back
        // into the program and would be glanceable. Basically, I think in the
        // ideal world we'd want to accept JSON files with aspect ratios and
        // resolutions fully specified - as long as those values don't conflict
        // with each other.
    }

    // World
    let world = serde_json::from_reader(BufReader::new(File::open(config.world_path.unwrap())?))?;

    // Camera
    let cam = Camera::new(f64::from(aspect_ratio));

    // Header
    writeln!(output, "P3")?;
    writeln!(output, "{} {}", res.width, res.height)?;
    writeln!(output, "255")?;

    let image = (0..res.height).into_par_iter().map(|j| {
        let mut rng = rand::thread_rng();
        stderr().flush().unwrap();

        let scanline = (0..res.width).into_iter().map(|i| {
            let mut pixel_color = Vec3::new(0.0, 0.0, 0.0);
            for _ in 0..config.samples_per_pixel {
                let random_u_component: f64 = rng.gen();
                let random_v_component: f64 = rng.gen();

                let u =
                    ((i as f64) + random_u_component) / ((res.width - 1) as f64);
                let v =
                    ((j as f64) + random_v_component) / ((res.height - 1) as f64);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, config.max_depth, &mut rng);
            }

            pixel_color
        }).collect::<Vec<Color>>();

        scanline
    }).collect::<Vec<Vec<Color>>>();
    for scanline in image.iter().rev() {
        for pixel_color in scanline {
            write!(output, "{} ", pixel_color.format_color(config.samples_per_pixel))?;
        }
        writeln!(output)?;
    }
    eprintln!("\rDone!                          ");

    Ok(())
}
