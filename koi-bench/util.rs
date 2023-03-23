pub fn from_png(data: &[u8]) -> (Vec<u8>, (u32, u32, usize)) {
    let mut decoder = png::Decoder::new(data);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    let info = reader.info().clone();
    let channels = info.bytes_per_pixel();
    (buf, (info.width, info.height, channels))
}
