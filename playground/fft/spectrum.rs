use image::{ImageBuffer, RgbImage};
use ndarray::{Array2, Zip};
use num_complex::Complex;

use crate::{plot::visualize_frequencies, util};

pub fn run(file_name: &str) {
    let image = util::read_png(file_name);
    let (width, height) = image.dimensions();
    let mut denoised_img: RgbImage = ImageBuffer::new(width, height);

    for channel in 0..3 {
        // Transform the image into the frequency domain. This is a 2D array of complex numbers
        let fft_image: Array2<Complex<f32>> = util::fft_channel(&image, channel);

        visualize_frequencies(
            &fft_image,
            // red
            match channel {
                0 => (255, 0, 0),
                // green
                1 => (0, 255, 0),
                // blue
                2 => (0, 0, 255),
                _ => unreachable!(),
            },
            width,
            height,
            &format!("{}-{channel}.fft.png", file_name.replace(".png", "")),
        );

        let ifft_image = util::ifft_channel(&fft_image);

        for ((y, x), pixel) in ifft_image.indexed_iter() {
            denoised_img.get_pixel_mut(x as u32, y as u32)[channel] = *pixel;
        }
    }

    denoised_img
        .save(format!("{}.denoised.png", file_name))
        .unwrap();
}
