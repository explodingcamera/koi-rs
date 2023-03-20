use koi::{decode, encode, file::FileHeader, types::Compression};
use std::{fs::File, io::BufReader};

fn read_png(path: &str) -> (Vec<u8>, (u32, u32)) {
    let mut decoder = png::Decoder::new(File::open(path).unwrap());
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    let info = reader.info().clone();
    println!("{:?}", info);
    (buf, (info.width, info.height))
}

const CHANNELS: usize = 4;
const FILE: &str = "koi-cli/tests/bw-transparent.png";

pub fn run() {
    let (test_image, (width, height)) = read_png(FILE);
    let mut out = File::create("test.koi").expect("Failed to create file");

    println!("{}", test_image.len());

    let header = FileHeader::new(
        None,
        width,
        height,
        (CHANNELS as u8).try_into().unwrap(),
        Compression::None,
    );

    encode::<_, _, CHANNELS>(header, &test_image[..], &mut out).expect("Failed to encode");

    let encoded_file = BufReader::new(File::open("test.koi").expect("Failed to open file"));
    let mut decoded_file = Vec::with_capacity((width * height * (CHANNELS as u32)) as usize);
    let _header =
        decode::<_, _, CHANNELS>(encoded_file, &mut decoded_file).expect("Failed to decode");

    let out = File::create("test.png").expect("Failed to create file");
    let mut encoder = png::Encoder::new(out, width, height);

    if CHANNELS == 3 {
        encoder.set_color(png::ColorType::Rgb);
    } else {
        encoder.set_color(png::ColorType::Rgba);
    }

    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&decoded_file).unwrap();
    writer.finish().unwrap();
}
