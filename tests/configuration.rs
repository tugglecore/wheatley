use rand::prelude::*;
use std::path::{Component, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use wheatley::embed_assets;

fn read_fixture<T: AsRef<Path>>(filename: T) -> String {
    let mut file_path = PathBuf::from("./examples/books");
    file_path.push(filename);
    dbg!(&file_path);

    let file_path = file_path.canonicalize().unwrap();
    std::fs::read_to_string(file_path).unwrap()
}

fn read_random_book(separator: &str) -> (String, String) {
    let entry = WalkDir::new("examples/books")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect::<Vec<DirEntry>>()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone();

    (
        entry
            .path()
            .components()
            .filter_map(|c| match c {
                Component::Normal(part) => part.to_str(),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(separator),
        std::fs::read_to_string(entry.path()).unwrap(),
    )
}

fn confirm_book_in_library<T: AsRef<Path>>(filename: T) -> bool {
    let mut file_path = PathBuf::from("./examples/books");
    file_path.push(filename);
    std::fs::exists(file_path).unwrap()
}

#[test]
fn read_assets_from_location() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt").unwrap();

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[test]
fn read_loaction_given_windows_path() {
    let expected_contents = read_fixture("hardly_haunted.txt");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples\\books",
    };
    let file = wheatley.get("examples/books/hardly_haunted.txt").unwrap();

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[cfg(windows)]
#[test]
fn read_location_with_backslash_separator() {
    let (target_path, expected_contents) = read_random_book(r"\");

    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
        use_backslash_in_keys: true
    };
    let file = wheatley.get(target_path).unwrap();

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
    let file = wheatley.get("/examples/books/hardly_haunted.txt").unwrap();

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

#[test]
fn pre_slash_missing_without_prepend_option() {
    let (target_path, expected_contents) = read_random_book("/");

    dbg!(&target_path);
    let wheatley: wheatley::Wheatley = embed_assets! {
        location: "examples/books",
    };
    let file = wheatley.get(target_path).unwrap();

    assert_eq!(
        String::from_utf8(file.contents.to_vec()).unwrap(),
        expected_contents
    )
}

// #[test]
// fn include_globs() {
//     let expected_contents = read_fixture("theodor_seuss_geisel/dr_seuss/cat_in_the_hat.txt");
//
//     let wheatley: wheatley::Wheatley = embed_assets! {
//         location: "examples/books",
//         include_globs: ["./examples/**/dr_seuss/**"]
//     };
//     let file = wheatley
//         .get("examples/books/theodor_seuss_geisel/dr_seuss/cat_in_the_hat.txt")
//         .unwrap();
//
//     dbg!(&wheatley);
//
//     assert_eq!(
//         String::from_utf8(file.contents.to_vec()).unwrap(),
//         expected_contents
//     )
// }
