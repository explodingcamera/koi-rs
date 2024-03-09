use image::{ImageBuffer, RgbImage};
use ndarray::Array2;
use num_complex::Complex;

use crate::util::{self, reduce_high_frequency_noise};

pub fn run(file_name: &str) {
    let image = util::read_png(file_name);
    let (width, height) = image.dimensions();
    let mut denoised_img: RgbImage = ImageBuffer::new(width, height);

    for channel in 0..3 {
        // Transform the image into the frequency domain. This is a 2D array of complex numbers
        let mut fft_image: Array2<Complex<f32>> = util::fft_channel(&image, channel);

        reduce_high_frequency_noise(&mut fft_image, 340.0);

        let ifft_image = util::ifft_channel(&fft_image);

        for ((y, x), pixel) in ifft_image.indexed_iter() {
            denoised_img.get_pixel_mut(x as u32, y as u32)[channel] = *pixel;
        }
    }

    denoised_img
        .save(format!("{}.denoised.png", file_name))
        .unwrap();
}
