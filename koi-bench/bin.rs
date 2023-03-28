use std::hint::black_box;
use std::{collections::HashMap, io, time::Instant};
use strum::IntoEnumIterator;

use formats::ImageFormatType;

mod formats;
mod suite;
mod util;

use crate::suite::{generate_test_suites, FormatResult, Test};
use crate::util::from_png;

// how many times to run each test (to get the minimum time)
static RUNS: usize = 2;

fn main() -> io::Result<()> {
    let mut suites = generate_test_suites("images");

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

    for format in ImageFormatType::iter() {
        let mut format_impl = match channels {
            3 => format.get_impl::<3>(),
            4 => format.get_impl::<4>(),
            _ => {
                println!("Unsupported channel count: {}", channels);
                continue;
            } // TODO: images/textures_pk/pkw_wall11e.png
        };

        // ENCODE
        println!("encoding {format}...");
        let mut shortest_encode: u128 = 0;
        let mut output = vec![0; input.len()];
        let start = Instant::now();
        for _ in 0..RUNS {
            black_box(format_impl.encode(input, &mut output, (width, height))).unwrap_or_else(
                |e| {
                    println!("Error encoding {format}: {e}");
                },
            );

            let duration = start.elapsed().as_micros();
            shortest_encode = std::cmp::min(shortest_encode, duration)
        }
        let encode_size = output.len();

        // DECODE (TODO: broken)
        println!("decoding {format}...");
        let mut shortest_decode: u128 = 0;
        let start = Instant::now();
        let mut decode_output = vec![0; input.len()];
        for _ in 0..RUNS {
            format_impl
                .decode(&output, &mut decode_output, (width, height))
                .unwrap();
            let duration = start.elapsed().as_micros();
            shortest_decode = std::cmp::min(shortest_decode, duration)
        }

        results.insert(
            format,
            FormatResult {
                decode_min_time: 0,
                encode_min_time: shortest_encode,
                encode_size,
            },
        );
    }

    results
}
