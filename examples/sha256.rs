use sha2::{Sha256, Digest};
use drunken_diver::{Route, Dive};

fn main() {
    let mut hash = Sha256::new();
    hash.update(b"hey");
    let result = hash.finalize();

    println!("{}", Route::<16>::from(Dive::from(result[..].iter().map(|x| *x))));
}
