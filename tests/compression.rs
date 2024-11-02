#[cfg(any(feature = "br", feature = "gzip", feature = "snap", feature = "zstd"))]
use std::path::{Path, PathBuf};
use wheatley::embed_assets;

#[cfg(any(feature = "br", feature = "gzip", feature = "snap", feature = "zstd"))]
fn read_fixture<T: AsRef<Path>>(filename: T) -> String {
    let mut file_path = PathBuf::from("./examples/books");
    file_path.push(filename);
    let file_path = file_path.canonicalize().unwrap();
    std::fs::read_to_string(file_path).unwrap()
}

#[test]
#[cfg(feature = "br")]
fn br_compression() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut result = vec![];
    let mut contents = std::io::Cursor::new(Vec::from(file.contents));
    brotli::BrotliDecompress(&mut contents, &mut result).unwrap();
    let result = String::from_utf8(result).unwrap();

    assert_eq!(expected_contents, result);
}

#[test]
#[cfg(feature = "gzip")]
fn gzip_compression() {
    use std::io::prelude::*;
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };

    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut decoder = flate2::read::GzDecoder::new(file.contents);
    let mut result = String::new();
    decoder.read_to_string(&mut result).unwrap();

    assert_eq!(expected_contents, result);
}

#[test]
#[cfg(feature = "zstd")]
fn zstd_compression() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let contents = std::io::Cursor::new(Vec::from(file.contents));
    let result = zstd::decode_all(contents).unwrap();
    let result = String::from_utf8(result).unwrap();

    assert_eq!(expected_contents, result);
}

#[test]
#[cfg(feature = "snap")]
fn snap_compression() {
    use std::io::prelude::*;
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut result = vec![];
    snap::read::FrameDecoder::new(file.contents)
        .read_to_end(&mut result)
        .unwrap();
    let result = String::from_utf8(result).unwrap();

    assert_eq!(expected_contents, result);
}

#[test]
fn br_compression_absent_without_feature_flag() {
    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut result = vec![];
    let mut contents = std::io::Cursor::new(Vec::from(file.contents));

    assert!(matches!(
        brotli::BrotliDecompress(&mut contents, &mut result),
        Err(_)
    ))
}

#[test]
fn gzip_compression_absent_without_feature_flag() {
    use std::io::prelude::*;

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };

    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut decoder = flate2::read::GzDecoder::new(file.contents);
    let mut result = String::new();

    assert!(matches!(decoder.read_to_string(&mut result), Err(_)));
}

#[test]
fn snap_compression_absent_without_feature_flag() {
    use std::io::prelude::*;

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let mut result = vec![];

    assert!(matches!(
        snap::read::FrameDecoder::new(file.contents).read_to_end(&mut result),
        Err(_)
    ));
}

#[test]
fn zstd_compression_absent_without_feature_flag() {
    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    let contents = std::io::Cursor::new(Vec::from(file.contents));

    assert!(matches!(zstd::decode_all(contents.clone()), Err(_)));
}
