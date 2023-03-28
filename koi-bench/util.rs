pub fn from_png(data: &[u8]) -> (Vec<u8>, (u32, u32, usize)) {
    let mut options = png::DecodeOptions::default();
    options.set_ignore_crc(true);
    let mut decoder = png::Decoder::new_with_options(data, options);
    decoder.set_transformations(png::Transformations::EXPAND);

    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    let info = reader.info().clone();
    let channels = info.bytes_per_pixel();
    (buf, (info.width, info.height, channels))
}

pub fn to_dir(path: &str) -> String {
    path.split('/')
        .take(path.split('/').count() - 1)
        .collect::<Vec<&str>>()
        .join("/")
}
