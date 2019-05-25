//! Interpreter for the hana programming language

#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![feature(core_intrinsics)]

#[macro_use] extern crate decorator;

pub mod compiler;
pub mod ast;
pub mod hanayo;
pub mod vmbindings;