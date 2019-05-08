#[macro_use] pub extern crate decorator;
#[macro_use] pub extern crate lazy_static;

pub mod gc;
pub mod compiler;
pub mod ast;
pub mod hanayo;
mod vmbindings;
pub use vmbindings::vm;