use std::hint::black_box;
use std::io::Cursor;
use std::{collections::HashMap, io, time::Instant};
use strum::IntoEnumIterator;

use formats::ImageFormatType;

mod formats;
mod suite;
mod util;

use crate::suite::{generate_test_suites, FormatResult, Test};
use crate::util::from_png;

// how many times to run each test (to get the minimum time)
static RUNS: usize = 3;

fn main() -> io::Result<()> {
    let mut suites = generate_test_suites("testsuite");

    for suite in suites.values_mut() {
        println!("Running tests for {}", suite.name);

        for file in suite.files.iter() {
            println!("--- {}", file);

            let input = std::fs::read(file)?;
            let (input, (width, height, channels)) = from_png(&input);

            suite.tests.push(Test {
                input_size: input.len(),
                name: file.to_string(),
                results: run_test(&input, width, height, channels),
            });
        }
    }

    println!("{:#?}", suites);

    Ok(())
}

fn run_test(
    input: &[u8],
    width: u32,
    height: u32,
    channels: usize,
) -> HashMap<ImageFormatType, FormatResult> {
    let mut results: HashMap<ImageFormatType, FormatResult> = HashMap::new();

    if channels != 3 && channels != 4 {
        println!("Unsupported number of channels");
        return results;
    }

    'outer: for format in ImageFormatType::iter() {
        // ENCODE
        println!("encoding {format}...");
        let mut shortest_encode: u128 = u128::MAX;
        let mut output = Vec::new();
        for r in 0..RUNS {
            let mut out = Vec::with_capacity(input.len());
            let mut encoder = format.get_impl_dyn(channels);
            let start = Instant::now();
            if let Err(e) = black_box(encoder.encode(input, &mut out, (width, height))) {
                println!("Error encoding {format}, skipping: {e}");
                continue 'outer;
            }
            shortest_encode = std::cmp::min(shortest_encode, start.elapsed().as_micros());

            if r == 0 {
                drop(encoder);
                output = out;
            }
        }

        println!("encoded {} bytes", output.len());
        let encode_size = output.len();

        // DECODE
        println!("decoding {format}...");
        let mut shortest_decode: u128 = u128::MAX;
        for _ in 0..RUNS {
            let data = Cursor::new(output.clone());

            let mut decoder = format.get_impl_dyn(channels);
            let start = Instant::now();

            if let Err(e) = black_box(decoder.decode(data, Vec::new(), (width, height))) {
                println!("Error decoding {format}, skipping: {e}");
                continue 'outer;
            }

            shortest_decode = std::cmp::min(shortest_decode, start.elapsed().as_micros())
        }

        results.insert(
            format,
            FormatResult {
                decode_min_time: shortest_decode,
                encode_min_time: shortest_encode,
                encode_size,
            },
        );
    }

    results
}
