use std::io::{self, Read};

use program_generation::generate;

pub(crate) mod expander;
pub mod inequalities;
pub mod program_generation;

fn main() {
    let mut stdin = io::stdin();
    let mut specification = String::new();
    if let Err(e) = stdin.read_to_string(&mut specification) {
        panic!("{}", e)
    }
    let generated_verification_code = generate(&specification);
    println!("{}", generated_verification_code);
}
