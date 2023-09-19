use std::io::{self, Read};

use program_generation::generate;

pub mod parsing;
pub mod program_generation;
pub mod systems;

fn main() {
    let mut stdin = io::stdin();
    let mut specification = String::new();
    if let Err(e) = stdin.read_to_string(&mut specification) {
        panic!("{}", e)
    }
    let generated_verification_code = generate(&specification);
    println!("{}", generated_verification_code);
}
