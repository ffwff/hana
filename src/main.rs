#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]

use std::io::Read;
use std::mem::ManuallyDrop;
#[macro_use] extern crate decorator;
extern crate ansi_term;
use ansi_term::Color as ac;

pub mod compiler;
pub mod ast;
mod vmbindings;
pub use vmbindings::vm;
pub use vmbindings::vm::VmOpcode;
pub use vmbindings::gc::set_root;
mod hanayo;

fn print_error<T>(s: &String, lineno: usize, col: usize, etype: &str, message: T)
    where T: std::fmt::Debug {
    let line = s.split("\n").nth(lineno-1).unwrap();
    let lineno_info = format!("{} | ", lineno);
    let lineno_info_len = lineno_info.len();
    eprintln!("
{}{}
{}

{} {:?}",
    ac::Blue.bold().paint(lineno_info),
    line,
    ac::Blue.bold().paint(" ".repeat(lineno_info_len + col-1) + "^"),
    ac::Red.bold().paint(etype.to_string() + ":"),
    message);
}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    let mut file = std::fs::File::open(&args[1]).unwrap_or_else(|err| {
        println!("error opening file: {}", err);
        std::process::exit(1);
    });
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap_or_else(|err| {
        println!("error reading file: {}", err);
        std::process::exit(1);
    });
    let prog = ast::grammar::start(&s).unwrap_or_else(|err| {
        print_error(&s, err.line, err.column, "parser error", err.expected);
        std::process::exit(1);
    });
    let mut c = ManuallyDrop::new(compiler::Compiler::new());
    // manually drop because the interpreter will exit early
    for stmt in prog {
        stmt.emit(&mut c);
    }
    set_root(&mut c.vm);
    hanayo::init(&mut c.vm);
    c.vm.code.push(VmOpcode::OP_HALT);
    c.vm.execute();
}