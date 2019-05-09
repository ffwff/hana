#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate decorator;
use std::io::Read;

pub mod compiler;
pub mod ast;
mod vmbindings;
pub use vmbindings::vm;
pub use vmbindings::vm::VmOpcode;
pub use vmbindings::gc::add_root;
mod hanayo;

fn main() {
    let args : Vec<String> = std::env::args().collect();
    let mut file = match std::fs::File::open(&args[1]) {
        Err(e) => { panic!("Error opening file: {}", e); }
        Ok(f) => f
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(e) => { panic!("Error reading file: {}", e); }
        Ok(_) => { }
    };
    let prog = ast::grammar::start(&s).unwrap();
    //println!("{:?}", prog); return;
    let mut c = compiler::Compiler::new();
    for stmt in prog {
        stmt.emit(&mut c);
    }
    add_root(&mut c.vm);
    hanayo::init(&mut c.vm);
    c.vm.code.push(VmOpcode::OP_HALT);
    c.vm.execute();
}