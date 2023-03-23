use std::{collections::HashMap, io, time::Instant};
use strum::IntoEnumIterator;

use formats::{ImageFormatType, ImageFormat};
use util::from_png;
use walkdir::WalkDir;
mod formats;
mod util;

static RUNS: usize = 10;

#[derive(Debug)]
struct FormatResult {
    pub decode_min_time: u128,
    pub encode_min_time: u128,
    pub encode_size: usize,
}

#[derive(Debug)]
struct TestSuite {
    pub name: String,
    pub files: Vec<String>,
    pub tests: Vec<Test>,
}

#[derive(Debug)]
struct Test {
    pub name: String,
    pub input_size: usize,
    pub results: HashMap<ImageFormatType, FormatResult>,
}

fn main() -> io::Result<()> {
    let mut suites: HashMap<String, TestSuite> = HashMap::new();

    for entry in WalkDir::new("images") {
        let Ok(entry) = entry else {
            continue; 
        };

        let Some(path) = entry.path().to_str() else {
            continue;
        };

        if path == "images" || path.contains('.') && !path.ends_with(".png") {
            continue;
        }

        if entry.file_type().is_dir() {
            suites.insert(
                path.to_string(),
                TestSuite {
                    name: path.to_string(),
                    files: Vec::new(),
                    tests: Vec::new(),
                },
            );

            continue;
        }

        let Some(suite) = suites.get_mut(&to_dir(path)) else {
            println!("No suite for {}", path);
            continue;
        };

        suite.files.push(path.to_string());
    }


    for suite in suites.values_mut() {
        println!("Running tests for {}", suite.name);

        for file in suite.files.iter() {
            println!("--- {}", file);

            let input = std::fs::read(file)?;
            let (
                input,
                (width, height, channels),
            ) = from_png(&input);

            let results =  match channels {
                3 => run_test::<3>( &input, width, height),
                4 => run_test::<4>( &input, width, height),
                _ => panic!("Unsupported channel count: {}", channels) // TODO: images/textures_pk/pkw_wall11e.png
            };

            suite.tests.push(Test {
                input_size: input.len(),
                name: file.to_string(),
                results,
            });
        }
    }

    println!("{:#?}", suites);

    Ok(())
}

fn run_test<const C: usize>(
    input: &[u8],
    width: u32,
    height: u32,
) -> HashMap<ImageFormatType, FormatResult> {
    let mut results: HashMap<ImageFormatType, FormatResult> = HashMap::new();

    for format in ImageFormatType::iter() {
        // ENCODE
        println!("encoding {format}...");
        let mut shortest_encode: u128 = 0;
        let mut output = vec![0; input.len()];
        let start = Instant::now();
        for _ in 0..RUNS {
            match format {
                ImageFormatType::KOI => {
                    formats::koi::Koi::new().encode::<C>(input, &mut output, (width, height)).unwrap();                            
                }
                ImageFormatType::PNG => {
                    formats::png::Png::new().encode::<C>(input, &mut output, (width, height)).unwrap();
                }
                _ => {}
            }

            let duration = start.elapsed().as_micros();
            shortest_encode = std::cmp::min(shortest_encode, duration)
        }
        let encode_size = output.len();
        
        // // DECODE (TODO: broken)
        // println!("decoding {format}...");
        // let mut output = vec![0; input.len()];
        // let mut shortest_decode: u128 = 0;
        // let start = Instant::now();
        // for _ in 0..RUNS {
        //     match format {
        //         ImageFormatType::KOI => {
        //             formats::koi::Koi::new().decode::<C>(input, &mut output, (width, height)).unwrap();
        //         }
        //         ImageFormatType::PNG => {
        //             formats::png::Png::new().decode::<C>(input, &mut output, (width, height)).unwrap();
        //         }
        //         _ => {}
        //     }
        //     let duration = start.elapsed().as_micros();
        //     shortest_decode = std::cmp::min(shortest_decode, duration)
        // }

        results.insert(format, 
            FormatResult {
                // decode_min_time: shortest_decode,
                decode_min_time: 0,
                encode_min_time: shortest_encode,
                encode_size,
            }
        );
    }

    results
}

fn to_dir(path: &str) -> String {
    path.split('/')
        .take(path.split('/').count() - 1)
        .collect::<Vec<&str>>()
        .join("/")
}
