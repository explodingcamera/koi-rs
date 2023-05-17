mod fft;
mod plot;
mod util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args.get(1).expect("missing program name");

    match program.as_str() {
        "fft-spectrum" => fft::spectrum::run(args.get(2).expect("missing file name")),
        "fft-noise" => fft::noise::run(args.get(2).expect("missing file name")),
        _ => panic!("unknown program"),
    }
}
