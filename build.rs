use std::env;
extern crate cc;
extern crate peg;

fn main() {
    let is_release = env::var("PROFILE").unwrap() == "release";

    // parser
    peg::cargo_build("src/parser.peg");

    // cc
    let mut build = cc::Build::new();
    for path in std::fs::read_dir("./src/vm").unwrap() {
        if let Some(path) = path.unwrap().path().to_str() {
            let pstr = path.to_string();
            if pstr.ends_with(".c") {
                build.file(pstr);
            }
        }
    }
    if env::var("NOLOG").is_ok() || is_release {
        build.define("NOLOG", None); }
    if env::var("PROFILE").is_ok() {
        build.flag("-pg"); }
    if is_release { build.flag("-flto"); }
    build
        .flag("-Wall").flag("-Werror")
        .compile("hana");
}