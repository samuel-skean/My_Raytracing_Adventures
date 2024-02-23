#include <iostream>

int main() {

    // Image

    const int image_width = 256;
    const int image_height = 256;

    // Header

    std::cout << "P3\n" << image_width << ' ' << image_height << "\n255\n";

    for (int j = 0; j < image_height; ++j) {
        for (int i = 0; i < image_width; ++i) {
            double r = double(i) / (image_width - 1);
            double g = double(j) / (image_height - 1);
            double b = 0;

            int ir = static_cast<int>(255.999 * r);
                // I think I get the 255.999 - it would be really bad if we ever
                // produced a 256 subpixel value (invalid-file bad) and the precision
                // of floating point numbers is not perfect. But that doesn't seem
                // to ever happen for me.
            int ig = static_cast<int>(255.999 * g);
            int ib = static_cast<int>(255.999 * b);

            std::cout << ir << ' ' << ig << ' ' << ib << ' ';
        }
        std::cout << '\n';
    }
}