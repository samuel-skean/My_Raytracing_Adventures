mod vec;
mod ray;
mod hit;
mod sphere;
mod camera;
mod material;
mod plane;
mod gui;

use std::{fs::File, io::{self, stderr, BufReader, BufWriter, Write}, thread, time::Duration};
use atomic::Atomic;
use clap_serde_derive::{clap::{self, error::ErrorKind, CommandFactory as _, Parser}, ClapSerde};
use gui::run_gui;
use serde::{Serialize, Deserialize};
use rand::{Rng, SeedableRng};

use rand_chacha::ChaCha12Rng;
use vec::Color;
use ray::Ray;
use hit::{Hit, World};
use camera::Camera;

const DEFAULT_NUM_THREADS: usize = 8;

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
            rec.mat.emit(rng, r, &rec) + attenuation * ray_color(&scattered, world, depth - 1, rng)
        }
        else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        // Color::new(0.0, 0.0, 0.0)
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

#[derive(Debug, Serialize, Deserialize, ClapSerde, Clone)]
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
    image_width: Option<usize>,
    /// Image height [default: 144]
    #[arg(short = 'H', long)]
    image_height: Option<usize>,
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
    #[arg(short = 't', long)]
    num_threads: usize,
    /// Only required if no config is specified.
    #[arg(required_unless_present("config_path"))]
    world_path: Option<std::path::PathBuf>,
}

#[derive(Clone, Copy)]
struct Resolution {
    width: usize,
    height: usize,
}

fn get_aspect_ratio_and_resolution(aspect_ratio: Option<f64>, width: Option<usize>, height: Option<usize>) -> (f64, Resolution) {
    const DEFAULT_HEIGHT: usize = 144;
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
            height: (width as f64 / calculated_aspect_ratio) as usize,
        } ),
        (_, None, Some(height)) => (calculated_aspect_ratio, Resolution {
            width: (calculated_aspect_ratio * height as f64) as usize,
            height,
        }),
        (_, None, None) => (calculated_aspect_ratio, Resolution {
            width: (calculated_aspect_ratio * DEFAULT_HEIGHT as f64) as usize,
            height: DEFAULT_HEIGHT,
        }),
    }
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct PixelInfo {
    accumulated_color: Color,
    samples_so_far: u64,
}

impl Default for PixelInfo {
    fn default() -> Self {
        Self {
            accumulated_color: Color::new(0.0, 0.0, 0.0),
            samples_so_far: 0,
        }
    }
}

type PixelGrid = Vec<Vec<Atomic<PixelInfo>>>;
fn main() -> io::Result<()> {
    let default_config = Config {
        aspect_ratio: None,
        image_width: None,
        image_height: None,
        samples_per_pixel: 100,
        max_depth: 5,
        output_path: None,
        random_seed: 0,
        num_threads: DEFAULT_NUM_THREADS,
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
            Box::new(BufWriter::new(File::create(output_path).unwrap()))
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
    let world = serde_json::from_reader(BufReader::new(File::open(config.world_path.as_ref().unwrap()).unwrap())).unwrap();

    // Camera
    let cam = Camera::new(f64::from(aspect_ratio));

    let mut image: PixelGrid = PixelGrid::new();
    image.resize_with(res.height as usize, || {
        let mut row = Vec::new();
        row.resize_with(res.width as usize, Default::default);
        row
    });

    thread::scope(|s| {
        for thread_num in 0..config.num_threads {
            let config = &config;
            let image = &image;
            let world = &world;
            let cam = &cam;
            s.spawn(move || {

                // For more info on ANSI codes:
                // https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
                // TODO: Make the cursor not blink! (Probably by printing the escape
                // codes before and after the entire duration of the threads). The
                // escape code listed on the above gist doesn't seem to work on
                // either macOS Terminal or VSCode's Terminal.
                let offset_ansi_code = format!("\x1B[{}G", thread_num * 7 + 1);
                let height_of_portion = res.height / config.num_threads;

                let starting_height = thread_num * height_of_portion;
                let ending_height = (thread_num + 1) * height_of_portion;

                println!("Thread {thread_num} - Starting height: {starting_height:4}, Ending height: {ending_height:4}");
                thread::sleep(Duration::from_millis(200));
    
                let mut rng = ChaCha12Rng::seed_from_u64(config.random_seed);
                for s in 0..config.samples_per_pixel {
                    eprint!("\r{}{:4}", offset_ansi_code, config.samples_per_pixel - s);
                    for j in starting_height..ending_height {
                        stderr().flush().unwrap();

                        for i in 0usize..res.width {
                            
                            let random_u_component: f64 = rng.gen();
                            let random_v_component: f64 = rng.gen();

                            let u =
                                ((i as f64) + random_u_component) / ((res.width - 1) as f64);
                            let v =
                                (((res.height - j) as f64) + random_v_component) / ((res.height - 1) as f64);

                            let r = cam.get_ray(u, v);
                            let old_pixel_info = image[j][i].load(atomic::Ordering::Acquire);
                            let accumulated_color = old_pixel_info.accumulated_color + ray_color(&r, &world, config.max_depth, &mut rng);
                            image[j][i].store(PixelInfo { accumulated_color, samples_so_far: old_pixel_info.samples_so_far + 1 }, atomic::Ordering::Release);
                        }
                    }
                }

                eprint!("\r{}Done!", offset_ansi_code);
            });
        }

        run_gui(&image, res);
    });

    // Header
    writeln!(output, "P3")?;
    writeln!(output, "{} {}", res.width, res.height)?;
    writeln!(output, "255")?;

    for scanline in image.iter() {
        for pixel_color in scanline {
            write!(output, "{} ", pixel_color.load(atomic::Ordering::Acquire)
                .accumulated_color.format_color(config.samples_per_pixel))?;
        }
        writeln!(output)?;
    }

    eprintln!(); // Print newline, to keep around final "Done!" messages.

    Ok(())
}
