fn main() {
    // Image:

    const IMAGE_WIDTH: u16 = 256;
    const IMAGE_HEIGHT: u16 = 256;


    // Header:

    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in 0..IMAGE_WIDTH {
        for i in 0..IMAGE_HEIGHT {
            let r = i;
            let g = j;
            let b = 0;

            print!("{r} {g} {b} ");
        }
        println!();
    }
}
