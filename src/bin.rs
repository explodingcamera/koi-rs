use std::io::{BufReader, Read, Write};

use koi::{
    file::FileHeader,
    types::{Channels, Compression},
};

fn main() {
    let mut bytes = vec![];

    let header = FileHeader::new(12, None, 1, 1, Channels::Gray, Compression::Lz4);
    header.write(&mut bytes).unwrap();
    bytes.write_all(b"Hello, world!").unwrap();

    let reader = &mut bytes.as_slice();
    let mut buffer = BufReader::new(reader);

    let header = FileHeader::read(&mut buffer).unwrap();
    println!("{:?}", header);

    let mut buf = Vec::new();
    buffer.read_to_end(&mut buf).unwrap();
}
