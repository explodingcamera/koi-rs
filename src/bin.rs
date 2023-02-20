use std::io::{BufReader, Read, Write};

use koi::file::FileHeader;

fn main() {
    let mut bytes = vec![];

    let header = FileHeader::new(1, None);
    header.write(&mut bytes).unwrap();
    bytes.write_all(b"Hello, world!").unwrap();

    let reader = &mut bytes.as_slice();
    let mut buffer = BufReader::new(reader);

    let header = FileHeader::read(&mut buffer).unwrap();
    println!("{:?}", header);

    let mut buf = Vec::new();
    buffer.read_to_end(&mut buf).unwrap();

    // print buf as hex
    println!("{:?}", hex::encode(buf));
}
