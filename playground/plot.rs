use image::{ImageBuffer, Rgb};
use ndarray::Array2;
use num_complex::Complex;

pub fn visualize_frequencies(
    frequencies: &Array2<Complex<f32>>,
    color: (u8, u8, u8),
    height: u32,
    width: u32,
    filename: &str,
) {
    // Compute the magnitudes of the complex values and apply log transformation
    let magnitudes = frequencies.mapv(|c| (c.norm() + 1.0).ln());

    // Find the maximum magnitude (this will be used for normalization)
    let max_magnitude = magnitudes.fold(f32::MIN, |n, ac| f32::max(n, *ac));

    // Create a new image buffer
    let mut img = ImageBuffer::new(width, height);

    // Iterate over the pixels of the image
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Get the corresponding frequency magnitude
        let mag = magnitudes[(x as usize, y as usize)];

        // Normalize the magnitude to the range 0-255
        let normalized_mag = ((mag / max_magnitude) * 255.0) as u8;

        // Set the pixel color, using the normalized magnitude to modulate the provided color
        *pixel = Rgb([
            (color.0 as f32 * normalized_mag as f32 / 255.0) as u8,
            (color.1 as f32 * normalized_mag as f32 / 255.0) as u8,
            (color.2 as f32 * normalized_mag as f32 / 255.0) as u8,
        ]);
    }

    // Save the image
    img.save(filename).unwrap();
}
