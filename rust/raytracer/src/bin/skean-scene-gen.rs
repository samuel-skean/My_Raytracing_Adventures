use std::sync::Arc;

use clap_serde_derive::clap::{self, Parser};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use skean_raytracer::{
    hit::World, material::{Lambertian, Material, Metal}, plane::Plane, sphere::Sphere, vec::{Color, Point3, Vec3}
};

#[derive(Parser)]
struct Cli {
    /// Random seed to use to generate the scene.
    #[arg(short = 'R', long, default_value_t = 0)]
    random_seed: u64,
    /// Whether or not to allow collision between generated objects.
    #[arg(short = 'C', long, default_value_t = false)]
    allow_collision: bool,
    /// How many spheres to generate.
    #[arg(short = 'S', long, default_value_t = 200)]
    num_spheres: u64,
    /// How many planes to generate.
    #[arg(short = 'P', long, default_value_t = 5)]
    num_planes: u64,
    /// Probability, between 0 and 1, that a given object is metallic. The
    /// alternative is that it is diffuse.
    #[arg(short = 'm', long, default_value_t = 0.6)]
    metallic_probability: f64,
    // Probability, between 0 and 1, that a diffuse material is emissive.
    #[arg(long, alias = "ed", default_value_t = 0.8)]
    emissive_probability_diffuse: f64,
    /// Probability that a metallic material is emissive.
    #[arg(long, alias = "em", default_value_t = 0.2)]
    emissive_probability_metallic: f64,
    /// Probability, between 0 and 1, that a given sphere is small (of radius
    /// between 0.0 and 0.4). The alternative is that it is large (of radius
    /// between 15.0 and 20.0).
    #[arg(short = 's', long, default_value_t = 0.9)]
    small_sphere_probability: f64,
}

fn gen_material(options: &Cli, rng: &mut impl Rng) -> Arc<dyn Material> {
    let rand_color = Color::new(rng.gen(), rng.gen(), rng.gen());
    let rand_mat: Arc<dyn Material> = if rng.gen_bool(options.metallic_probability) {
        if rng.gen_bool(options.emissive_probability_metallic) {
            let rand_emission = Color::new(rng.gen(), rng.gen(), rng.gen());
            Arc::new(Metal::new_emissive(rand_color, rng.gen(), rand_emission))
        } else {
            Arc::new(Metal::new(rand_color, rng.gen()))
        }
    } else {
        if rng.gen_bool(options.emissive_probability_diffuse) {
            let rand_emission = Color::new(rng.gen(), rng.gen(), rng.gen());
            Arc::new(Lambertian::new_emissive(rand_color, rand_emission))
        } else {
            Arc::new(Lambertian::new(rand_color))
        }
    };

    rand_mat
}

fn main() {

    // TODO: There's gotta be a cleaner way to do this!! With less redundant code between things.
    
    let options = Cli::parse();
    let mut world = World::new();

    let mut rng = ChaCha12Rng::seed_from_u64(options.random_seed);
    for _ in 0..options.num_spheres {
        'getting_a_good_sphere: loop {
            let rand_mat = gen_material(&options, &mut rng);

            let sphere = if rng.gen_bool(options.small_sphere_probability) {
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
            if !options.allow_collision {
                for hit in world.iter() {
                    if hit.collides_with_sphere(&sphere) {
                        continue 'getting_a_good_sphere;
                    }
                }
            }
            world.push(Box::new(sphere));
            break 'getting_a_good_sphere;
        }
    }

    for _ in 0..options.num_planes {
        let rand_mat = gen_material(&options, &mut rng);

        let plane = Plane::new(
            Point3::new(
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..-25.0),
            ),
            Vec3::new(
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..-25.0),
            ),
            rand_mat,
        );
        world.push(Box::new(plane));
    }

    serde_json::to_writer_pretty(std::io::stdout(), &world).expect("Unable to write to standard out.");
}
