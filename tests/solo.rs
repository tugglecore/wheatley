use rand::prelude::*;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use tempfile::NamedTempFile;
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

// #[test]
// fn include_globs_return_correct_matches() {
//     let expected_contents = read_fixture("theodor_seuss_geisel/dr_seuss/cat_in_the_hat.txt");
//
//     let wheatley: wheatley::Wheatley = embed_assets! {
//         location: "examples/books",
//         include_globs: ["theodor_seuss_geisel/*"]
//     };
//     dbg!(&wheatley);
//     let file = wheatley
//         .get("examples/books/theodor_seuss_geisel/dr_seuss/cat_in_the_hat.txt")
//         .unwrap();
//
//
//     assert_eq!(
//         String::from_utf8(file.contents.to_vec()).unwrap(),
//         expected_contents
//     )
// }

// #[test]
// fn include_globs_will_not_return_unrequested_entry() {
//     assert!(
//         confirm_book_in_library("hug_a_cactus.txt")
//     );
//
//     let wheatley: wheatley::Wheatley = embed_assets! {
//         location: "examples/books",
//         include_globs: ["theordor_seuss/**"]
//     };
//     let file = wheatley
//         .get("examples/books/hug_a_cactus");
//
//     dbg!(&wheatley);
//     assert!(file.is_none());
// }
//
// #[test]
// fn exclude_globs_reject_matches() {
//     assert!(
//         confirm_book_in_library(
//             "suzanne_lang/grumpy_monkey.txt"
//         )
//     );
//
//     let wheatley: wheatley::Wheatley = embed_assets! {
//         location: "examples/books",
//         exclude_globs: ["suzanne_lang/**"]
//     };
//     let file = wheatley
//         .get("examples/books/suzanne_lang/grumpy_monkey.txt");
//
//     assert!(
//         file.is_none()
//     )
// }

// #[test]
// fn include_manifest_will_only_return_requested_entries() {
//     let expected_contents = read_fixture("deborah_diesen/pout_pout_fish.txt");
//     let mut wheatley_include_fixture = NamedTempFile::new_in("wheatley_include").unwrap();
//     writeln!(
//         wheatley_include_fixture,
//         "deborah_diesen/**",
//     ).unwrap();
//
//     let wheatley: wheatley::Wheatley = embed_assets! {
//         location: "examples/books"
//     };
//     let file = wheatley
//         .get("examples/books/deborah_diesen/pout_pout_fish.txt").unwrap();
//
//     assert_eq!(
//         String::from(file.contents.to_vec()),
//         expected_contents
//     );
// }
//
