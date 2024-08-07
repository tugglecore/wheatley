use hooser_derive::Hooser;

#[derive(Hooser)]
pub enum Cars {
    Civic,
    Corolla,
    Elantra,
    Sentra,
}

fn main() {
    let civic = Cars::Civic;
    let bio = civic.bio();
    println!("{bio}");

    let nissan = Cars::Sentra;
    let bio = nissan.bio();
    println!("{bio}");

    let corolla = Cars::Corolla;
    let bio = corolla.bio();
    println!("{bio}");
}
