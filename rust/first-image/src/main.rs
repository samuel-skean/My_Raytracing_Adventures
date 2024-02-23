mod vec;
use std::io::{stderr, Write};
use vec::{Vec3, Color};

fn main() {

    // Image:

    const IMAGE_WIDTH: u16 = 256;
    const IMAGE_HEIGHT: u16 = 256;


    // Header:

    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {:3}", j + 1);
        stderr().flush().unwrap();

        for i in 0..IMAGE_WIDTH {
            let pixel_color = Color::new((i as f64) / ((IMAGE_WIDTH - 1) as f64),
                                         (j as f64) / ((IMAGE_HEIGHT - 1) as f64),
                                        0.25);

            print!("{} ", pixel_color.format_color());
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}