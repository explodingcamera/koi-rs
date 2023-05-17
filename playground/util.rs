use image::{ImageBuffer, Rgb, RgbImage};
use ndarray::{s, Array2};
use num_complex::Complex;
use rustfft::FftPlanner;

pub fn read_png(path: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let res = image::open(path).expect("failed to open image");
    res.to_rgb8()
}

pub fn fft_channel(img: &RgbImage, channel: usize) -> Array2<Complex<f32>> {
    let (width, height) = img.dimensions();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(width as usize);

    let mut output = Array2::zeros((height as usize, width as usize));

    for y in 0..height {
        let mut input: Vec<Complex<f32>> = (0..width)
            .map(|x| Complex::new(img.get_pixel(x, y)[channel] as f32, 0.0))
            .collect();

        fft.process(&mut input);
        for x in 0..width as usize {
            output[[y as usize, x]] = input[x];
        }
    }

    output
}

pub fn ifft_channel(fft_output: &Array2<Complex<f32>>) -> Array2<u8> {
    let (height, width) = fft_output.dim();
    let mut planner = FftPlanner::new();
    let ifft = planner.plan_fft_inverse(width);

    let mut output = Array2::zeros((height, width));

    for y in 0..height {
        let mut input: Vec<Complex<f32>> = fft_output.slice(s![y, ..]).to_owned().into_raw_vec();
        ifft.process(&mut input);

        // Scaling the output by 1/N.
        input.iter_mut().for_each(|c| *c *= 1f32 / (width as f32));

        // Find the min and max for normalization
        let min = input.iter().map(|c| c.re).fold(f32::INFINITY, f32::min);
        let max = input.iter().map(|c| c.re).fold(f32::NEG_INFINITY, f32::max);

        for x in 0..width {
            // Normalize to the range 0..255
            let pixel_value = ((input[x].re - min) / (max - min) * 255.0)
                .max(0.0)
                .min(255.0) as u8;
            output[[y, x]] = pixel_value;
        }
    }

    output
}

pub fn reduce_high_frequency_noise(frequencies: &mut Array2<Complex<f32>>, threshold: f32) {
    // Compute the center of the array
    let (center_x, center_y) = (frequencies.dim().0 / 2, frequencies.dim().1 / 2);

    // Iterate over the frequency data
    for ((x, y), frequency) in frequencies.indexed_iter_mut() {
        // Compute the distance from the center
        let dx = x as isize - center_x as isize;
        let dy = y as isize - center_y as isize;
        let distance = ((dx * dx + dy * dy) as f32).sqrt();

        // If the distance is greater than the threshold, set the frequency to zero
        if distance > threshold {
            *frequency = Complex::new(0.0, 0.0);
        }
    }
}

#[derive(Copy, Clone)]
pub enum PassFilter {
    LowPass,
    HighPass,
}

pub fn pass_filter(fft_output: &mut Array2<Complex<f32>>, cutoff: f32, filter: PassFilter) {
    let (height, width) = fft_output.dim();
    let (center_x, center_y) = (width / 2, height / 2);

    // Apply window function
    apply_window_function(fft_output);

    for ((y, x), pixel) in fft_output.indexed_iter_mut() {
        let dist = (((x as i32 - center_x as i32).pow(2) + (y as i32 - center_y as i32).pow(2))
            as f32)
            .sqrt();

        if match filter {
            PassFilter::LowPass => dist > cutoff,
            PassFilter::HighPass => dist < cutoff,
        } {
            *pixel = Complex::new(0.0, 0.0);
        }
    }

    // Normalize image
    normalize_image(fft_output);
}

fn apply_window_function(fft_output: &mut Array2<Complex<f32>>) {
    let (height, width) = fft_output.dim();

    for ((y, x), pixel) in fft_output.indexed_iter_mut() {
        let window_x =
            0.5 * (1.0 - ((2.0 * std::f32::consts::PI * x as f32) / (width as f32)).cos());
        let window_y =
            0.5 * (1.0 - ((2.0 * std::f32::consts::PI * y as f32) / (height as f32)).cos());
        *pixel *= window_x * window_y;
    }
}

fn normalize_image(fft_output: &mut Array2<Complex<f32>>) {
    let max_value = fft_output
        .iter()
        .map(|pixel| pixel.norm_sqr())
        .fold(0.0_f32, |max, x| max.max(x))
        .sqrt();

    for pixel in fft_output.iter_mut() {
        *pixel /= max_value;
    }
}
