use std::rc::Rc;

use clap_serde_derive::clap::{self, Parser};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use skean_raytracer::{
    hit::World,
    material::{Lambertian, Metal, Scatter},
    sphere::Sphere,
    vec::{Color, Point3},
};

#[derive(Parser)]
struct Cli {
    /// Random seed to use to generate the scene.
    #[arg(short = 'R', long)]
    random_seed: u64,
}

fn main() {
    let options = Cli::parse();
    let mut world = World::new();

    let mut rng = ChaCha12Rng::seed_from_u64(options.random_seed);
    for _ in 0..200 {
        let rand_color = Color::new(rng.gen(), rng.gen(), rng.gen());
        let rand_mat: Rc<dyn Scatter> = if rng.gen_bool(0.6) {
            Rc::new(Metal::new(rand_color, rng.gen()))
        } else {
            if rng.gen_bool(0.8) {
                let rand_emission = Color::new(rng.gen(), rng.gen(), rng.gen());
                Rc::new(Lambertian::new_emissive(rand_color, rand_emission))
            } else {
                Rc::new(Lambertian::new(rand_color))
            }
        };
        let sphere = if rng.gen_bool(0.9) {
            Sphere::new(
                Point3::new(
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-0.5..1.0),
                    rng.gen_range(-2.0..-1.0),
                ),
                rng.gen_range(0.0..0.4),
                rand_mat,
            )
        } else {
            Sphere::new(
                Point3::new(
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..-25.0),
                ),
                rng.gen_range(15.0..20.0),
                rand_mat,
            )
        };
        world.push(Box::new(sphere));
    }

    serde_json::to_writer_pretty(std::io::stdout(), &world).expect("Unable to write to standard out.");
}
