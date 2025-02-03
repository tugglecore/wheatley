use cfg_if::cfg_if;
use std::path::PathBuf;

#[cfg(feature = "gzip")]
fn compress_with_gzip(asset: &mut Vec<u8>) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    let mut codec = GzEncoder::new(Vec::new(), Compression::default());
    codec.write_all(&asset).unwrap();
    codec.finish().unwrap()
}

#[cfg(feature = "br")]
fn compress_with_br(asset: &mut Vec<u8>) -> Vec<u8> {
    use brotli::enc::{BrotliCompress, BrotliEncoderParams};

    let mut buffer = vec![];
    let mut w = asset.as_slice();
    BrotliCompress(&mut w, &mut buffer, &BrotliEncoderParams::default()).unwrap();

    buffer
}

#[cfg(feature = "zstd")]
fn compress_with_zstd(asset: &mut Vec<u8>) -> Vec<u8> {
    use zstd::stream::encode_all;

    encode_all(asset.as_slice(), 0).expect("Failed zstd compression")
}

#[cfg(feature = "snap")]
fn compress_with_snap(asset: &mut Vec<u8>) -> Vec<u8> {
    use snap::write::FrameEncoder;
    use std::io::prelude::*;

    let mut wtr = FrameEncoder::new(vec![]);
    wtr.write_all(asset).unwrap();
    wtr.into_inner().unwrap()
}

type Compressor = fn(&mut Vec<u8>) -> Vec<u8>;

pub fn get_compressor() -> Compressor {
    cfg_if! {
        if #[cfg(feature = "gzip")] {
            compress_with_gzip
        } else if #[cfg(feature = "br")] {
            compress_with_br
        } else if #[cfg(feature = "zstd")] {
            compress_with_zstd
        } else if #[cfg(feature = "snap")] {
            compress_with_snap
        } else {
            panic!("Program entered invalid state. Attempted to build compressor without codec feature.");
        }
    }
}

pub fn compress_assets(hash_table: &mut [(String, Vec<u8>)]) {
    let compressor = get_compressor();

    for (_, asset) in hash_table.iter_mut() {
        *asset = compressor(asset);
    }
}
