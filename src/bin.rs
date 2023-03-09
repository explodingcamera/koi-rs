use std::{
    fs,
    io::{BufReader, Read, Write},
};

use koi::{
    encoder::PixelEncoder,
    file::FileHeader,
    types::{Channels, Compression},
};

fn main() {
    let mut bytes = vec![];

    let header = FileHeader::new(12, None, 1, 1, Channels::Gray, Compression::Lz4);
    header.write(&mut bytes).unwrap();
    bytes.write_all(b"Hello, world!").unwrap();

    let pixel_count = 12 * 12;
    let mut f = fs::File::create("/tmp/foo").expect("Unable to create file");
    let f2 = fs::File::open("/etc/hosts").expect("Unable to open file");

    let mut encoder = PixelEncoder::<_, 3>::new_lz4(&mut f, pixel_count);
    encoder.encode(f2).unwrap();

    let reader = &mut bytes.as_slice();
    let mut buffer = BufReader::new(reader);

    let header = FileHeader::read(&mut buffer).unwrap();
    println!("{:?}", header);

    let mut buf = Vec::new();
    buffer.read_to_end(&mut buf).unwrap();
}
