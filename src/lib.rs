#[macro_use] pub extern crate decorator;

pub mod gc;
pub mod compiler;
pub mod ast;
pub mod hanayo;
mod vmbindings;
pub use vmbindings::vm;