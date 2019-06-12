use std::env;
extern crate peg;

fn main() {
    let is_release = env::var("PROFILE").unwrap() == "release";

    // parser
    peg::cargo_build("src/parser.peg");
}
