use std::path::{Path, PathBuf};
use wheatley::embed_assets;

fn read_fixture<T: AsRef<Path>>(filename: T) -> String {
    let mut file_path = PathBuf::from("./examples/books");
    file_path.push(filename);
    let file_path = file_path.canonicalize().unwrap();
    std::fs::read_to_string(file_path).unwrap()
}

#[test]
fn read_assets_from_location() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[test]
fn read_loaction_given_as_windows_path() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples\\books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[test]
fn prepend_slash() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
        prepend_slash: true
    };
    let file = wheatley.get("/examples/books/hardly_haunted.txt");

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[test]
fn pre_slash_missing_without_option() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt");

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}
