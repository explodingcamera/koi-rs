# Koi - Kinda OK Image Format

A simple image format based on and inspired by [QOI](https://qoiformat.org/) and [QOIR](https://nigeltao.github.io/blog/2022/qoir.html).

## Differences

- File header is encoded using BSON to allow for arbitrary metadata (might switch to CBOR later)
- Streaming API
- Supports additional lz4 compression

<!-- https://encode.su/threads/3753-QOI-(Quite-OK-Image-format)-lossless-image-compression-to-PNG-size -->
<!-- https://docs.rs/multiversion/latest/multiversion/ -->
