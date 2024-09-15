static WHEATLEY: wheatley::Wheatley = wheatley::embed_assets! {
    location: "examples/books",
    compression_algorithm: "br",
    compression_unit: "directory",
    file_check: [
        "REQUIRED_FILES",
        "examples/books/hardly_haunted"
    ]
    prepend_slash: true,
};

fn main() {
    println!("Hello, my name is Sam I am");

    let file = WHEATLEY.get("examples\\books\\hardly_haunted.txt");
    let file = std::str::from_utf8(file.contents).unwrap();
    println!("Here we go: {file:#?}");
}
