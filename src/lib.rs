#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]

#[macro_use] pub extern crate decorator;

pub mod compiler;
pub mod ast;
pub mod hanayo;
pub mod vmbindings; // TODO
pub use vmbindings::gc;
pub use vmbindings::vm;