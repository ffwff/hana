#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]

#[macro_use] pub extern crate decorator;
#[macro_use] pub extern crate lazy_static;

pub mod compiler;
pub mod ast;
pub mod hanayo;
mod vmbindings;
pub use vmbindings::gc;
pub use vmbindings::vm;