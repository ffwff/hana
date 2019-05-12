#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]

#[macro_use] extern crate decorator;
use std::io::Read;
use std::mem::ManuallyDrop;

pub mod compiler;
pub mod ast;
mod vmbindings;
pub use vmbindings::vm;
pub use vmbindings::vm::VmOpcode;
pub use vmbindings::gc::set_root;
mod hanayo;

fn main() {
    let args : Vec<String> = std::env::args().collect();
    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let prog = ast::grammar::start(&s).unwrap();
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