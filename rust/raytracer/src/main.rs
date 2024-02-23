mod vec;
mod ray;

use std::io::{stderr, Write};

use vec::{Vec3, Point3, Color};
use ray::Ray;

fn hit_sphere(center: Point3, radius: f64, r: &Ray) -> f64 {
    let oc = r.origin() - center; // A - C
    let a = r.direction().length().powi(2); // b . b
    let half_b = oc.dot(r.direction()); // 2b
    let c = oc.length().powi(2) - radius * radius; // (A - C) . (A - C) * r^2
    let quarter_discriminant = half_b * half_b - a * c;

    if quarter_discriminant < 0.0  {// there are no roots
        -1.0
    } else {
        (-half_b - quarter_discriminant.sqrt()) / a
            // the smaller of the two roots (smaller t, so 'closer' to the
            // camera - assuming nothing's behind us)
    }
}
// Gets a color from each ray that forms a gradient when put together in the
// viewport.
// Because the ray is normalized first, there is a slight horizontal gradient
// from light blue on the left, through white, and to light blue on the right.
// Basically, the x stole from the y when it was pointing left and pointing
// right. This is why the image is pretty :).
fn ray_color(r: &Ray) -> Color {
    let sphere_center = Point3::new(0.0, 0.0, -1.0);
    let t = hit_sphere(sphere_center, 0.5, r);

    if t > 0.0 {
        let n = (r.at(t) - sphere_center).normalized(); // The normal of the intersection.
        return 0.5 * Color::new(n.x() + 1.0, n.y() + 1.0, n.z() + 1.0);
            // Mapping the components of the normal, which have the range -1.0
            // to 1.0 as normalized vectors, to the range 0 to 1.0

    }

    let unit_direction = r.direction().normalized();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn main() {

    // Image:

    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u64 = 256;
    const IMAGE_HEIGHT: u64 = ((256 as f64) / ASPECT_RATIO) as u64;

    // Camera
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);

    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0
                                - Vec3::new(0.0, 0.0, focal_length);


    // Header:

    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {:3}", j + 1);
        stderr().flush().unwrap();

        for i in 0..IMAGE_WIDTH {
            let u = (i as f64) / ((IMAGE_WIDTH - 1) as f64);
            let v = (j as f64) / ((IMAGE_HEIGHT - 1) as f64);

            let r = Ray::new(origin,
                             lower_left_corner + u * horizontal + v * vertical - origin);
            let pixel_color = ray_color(&r);

            print!("{} ", pixel_color.format_color());
        }
        println!();
    }
    eprintln!("\rDone!                          ");
}