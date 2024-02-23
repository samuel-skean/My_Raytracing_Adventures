use std::io::{stderr, Write};

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
            let r = (i as f64) / ((IMAGE_WIDTH - 1) as f64);
            let g = (j as f64) / ((IMAGE_HEIGHT - 1) as f64);
            let b = 0.25;

            let ir = (255.999 * r) as u8;
            let ig = (255.999 * g) as u8;
            let ib = (255.999 * b) as u8;
            print!("{ir} {ig} {ib} ");
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}
