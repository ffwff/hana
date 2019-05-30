//! Interpreter for the hana programming language

#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate decorator;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate num_derive;

pub mod ast;
pub mod compiler;
pub mod hanayo;
pub mod vmbindings;
