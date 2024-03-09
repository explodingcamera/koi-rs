<p align="center">
    <img src="./koi.png" width="100px">
    <h1 align="center">Koi - The Kinda OK Image Format</h1>
</p>

<br/>

A simple image format based on and inspired by [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html).

A full overview of the differences and some benchmarks can be found in the accompanying [blog post](https://blog.henrygressmann.de/koi/).

<!-- https://encode.su/threads/3753-QOI-(Quite-OK-Image-format)-lossless-image-compression-to-PNG-size -->
<!-- https://docs.rs/multiversion/latest/multiversion/ -->

## Credits

- The [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html) formats
- A lot of great ideas from the [qoi2-bikeshed](https://github.com/nigeltao/qoi2-bikeshed/issues)

## Benchmarks

`cargo run --release --bin koi-bench`

# License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in OKV by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
