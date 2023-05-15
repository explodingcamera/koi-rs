use std::collections::BTreeMap;
use std::hint::black_box;
use std::{io, time::Instant};
use strum::IntoEnumIterator;

use formats::ImageFormatType;
use indicatif::ProgressBar;

mod formats;
mod suite;
mod util;

use crate::suite::{generate_test_suites, FormatResult, Test};
use crate::util::from_png;

// how many times to run each test (to get the minimum time)
static RUNS: usize = 1;

fn main() -> io::Result<()> {
    let mut suites = generate_test_suites("images");

    println!(
        " \x1b[1mRunning benchmarks\x1b[0m ({} runs per image)",
        RUNS
    );

    for suite in suites.values_mut() {
        if suite.files.is_empty() {
            continue;
        }

        // println!("=== Running tests for {suite_name}", suite_name = suite.name);
        // nicer unicode test name
        println!("┌──────────────────────────────────────────────────────┐");
        println!(
            "│ running tests for {suite_name: <34} │",
            suite_name = suite.name
        );
        println!("└──────────────────────────────────────────────────────┘");

        let pb = ProgressBar::new(suite.files.len() as u64);
        for file in suite.files.iter() {
            pb.inc(1);

            let input = std::fs::read(file)?;
            let (input, (width, height, channels)) = from_png(&input);

            let (results, errored) = run_test(&input, width, height, channels);

            suite.tests.push(Test {
                input_size: input.len(),
                name: file.to_string(),
                results,
                errored,
            });
        }
        pb.finish_and_clear()
    }

    // bold text saying "Results"
    println!("\n \x1b[1mResults\x1b[0m");

    for suite in suites.values() {
        let successfull_tests = suite
            .tests
            .iter()
            .filter(|t| !t.errored)
            .collect::<Vec<_>>();

        print_results(successfull_tests, &suite.name);
    }

    let all_tests = suites
        .values()
        .flat_map(|s| s.tests.iter())
        .filter(|t| !t.errored)
        .collect::<Vec<_>>();

    print_results(all_tests, "Overall");

    Ok(())
}

fn print_results(tests: Vec<&Test>, title: &str) {
    let total_input_size: usize = tests.iter().map(|t| t.input_size).sum();

    if total_input_size == 0 {
        return;
    }

    println!("┌─────────────────────────────────────┐");
    println!("│ {title: <35} │", title = title);
    println!("├─────────┬─────────┬─────────┬───────┤");
    println!("│ format  │ encode  │ decode  │ ratio │");
    println!("├─────────┼─────────┼─────────┼───────┤");
    for format in ImageFormatType::iter() {
        let total_size: usize = tests
            .iter()
            .map(|t| t.results.get(&format).unwrap().encode_size)
            .sum();

        let total_time_encode: u128 = tests
            .iter()
            .map(|t| t.results.get(&format).unwrap().encode_min_time)
            .sum();

        let total_time_decode: u128 = tests
            .iter()
            .map(|t| t.results.get(&format).unwrap().decode_min_time)
            .sum();

        println!(
            // "{format}: {size}kb - {encode}ms - {decode}ms - {compression_rate}",
            // only print the first 2 digits after the comma for compression rate
            "│ {format: <7} │ {encode: >5}ms │ {decode: >5}ms │ {compression_rate:>5.2} │",
            format = format,
            encode = total_time_encode / 1000,
            decode = total_time_decode / 1000,
            compression_rate = total_size as f64 / total_input_size as f64
        );
    }
    println!("└─────────┴─────────┴─────────┴───────┘");
}

fn run_test(
    input: &[u8],
    width: u32,
    height: u32,
    channels: usize,
) -> (BTreeMap<ImageFormatType, FormatResult>, bool) {
    let mut results: BTreeMap<ImageFormatType, FormatResult> = BTreeMap::new();
    let mut errored = false;

    if channels != 3 && channels != 4 {
        return (results, true);
    }

    'outer: for format in ImageFormatType::iter() {
        // ENCODE
        let mut shortest_encode: u128 = u128::MAX;
        let mut output = Vec::new();

        for r in 0..RUNS {
            let mut encoder = format.get_impl_dyn(channels);
            let start = Instant::now();

            let out = match black_box(encoder.encode(black_box(input), (width, height))) {
                Err(e) => {
                    println!("Error encoding {format}, skipping: {e}");
                    errored = true;
                    continue 'outer;
                }
                Ok(out) => out,
            };

            shortest_encode = std::cmp::min(shortest_encode, start.elapsed().as_micros());
            if r == 0 {
                output = out;
            }
        }
        let encode_size = output.len();

        // DECODE
        let mut shortest_decode: u128 = u128::MAX;
        for _ in 0..RUNS {
            let mut decoder = format.get_impl_dyn(channels);
            let start = Instant::now();

            if let Err(e) = black_box(decoder.decode(black_box(&output), (width, height))) {
                println!("Error decoding {format}, skipping: {e}");
                errored = true;
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

    (results, errored)
}
