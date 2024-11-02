static WHEATLEY: wheatley::Wheatley = wheatley::embed_assets! {
    location: "examples/books",
    file_check: [
        "REQUIRED_FILES",
        "examples/books/hardly_haunted"
    ]
    prepend_slash: true,
};

fn main() {
    println!("Hello, my name is Sam I am");

    let file = WHEATLEY.get("/examples/books/hardly_haunted.txt");
    // let mut d = GzDecoder::new(file.contents);
    // let mut s = String::new();
    // d.read_to_string(&mut s).unwrap();
    // println!("Here we go: {s:#?}");
}
