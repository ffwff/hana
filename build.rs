extern crate cc;
extern crate peg;

fn main() {
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
    build
        .flag("-Wall")
        .flag("-Werror")
        .define("NOLOG", None)
        .compile("hana");
}