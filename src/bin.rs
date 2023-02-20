use std::io::{BufReader, Read};

use bson::Document;
use koi::run;

fn main() {
    let bytes = hex::decode("0C0000001069000100000000").unwrap();
    let reader = &mut bytes.as_slice();
    let mut buffer = BufReader::new(reader);

    let doc = Document::from_reader(&mut buffer).unwrap();
    println!("{:?}", doc);

    let mut buf = Vec::new();
    buffer.read_to_end(&mut buf).unwrap();

    // print buf as hex
    println!("{:?}", hex::encode(buf));
}
