use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
};

use koi::{
    encoder::PixelEncoder,
    file::FileHeader,
    types::{Channels, Compression},
};
use png::Info;

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

pub fn run() {
    let (test_image, (width, height)) = read_png("koi-cli/tests/runlength.png");
    let mut out = File::create("test.koi").expect("Failed to create file");

    println!("{}", test_image.len());

    let header = FileHeader::new(None, width, height, Channels::Rgba, Compression::None);
    header.write(&mut out).expect("Failed to write header");

    let mut encoder = PixelEncoder::<_, 4>::new_uncompressed(&mut out, (width * height) as usize);
    encoder
        .encode(&*test_image)
        .expect("Failed to encode image");
    encoder.flush().expect("Failed to flush encoder");

    let in_file = File::open("test.koi").expect("Failed to open file");
    let mut in_file = BufReader::new(in_file);

    let header = FileHeader::read(&mut in_file).unwrap();
    println!("{:?}", header);

    let mut decoder = koi::decoder::PixelDecoder::<_, 4>::new_uncompressed(
        &mut in_file,
        (width * height) as usize,
    );

    let mut buf = vec![0; (width * height * 3) as usize];
    decoder.decode(&mut buf).unwrap();

    let mut out = File::create("test.png").expect("Failed to create file");
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&buf).unwrap();
    writer.finish().unwrap();

    // let reader = &mut bytes.as_slice();
    // let mut buffer = BufReader::new(reader);

    // let mut buf = Vec::new();
    // buffer.read_to_end(&mut buf).unwrap();
}
