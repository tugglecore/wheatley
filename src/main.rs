use hooser_derive::Hooser;

#[derive(Hooser)]
pub enum Poets {
    Lucille
}

fn main() {
    let lucille = Poets::Lucille;
    let bio = lucille.bio();
    println!("{bio}");
    
    println!("Hello, world!");
}
