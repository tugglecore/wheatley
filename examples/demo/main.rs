use wheatley::Hooser;

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
    println!("When was the first civic produced? {bio}");

    let elantra = Cars::Elantra;
    let maker = elantra.automaker();
    println!("Who makes Elantra? {maker}");
}
