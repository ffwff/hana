#[macro_use]
extern crate afl;
extern crate haru;

use haru::ast::grammar;
use haru::compiler;
use haru::hanayo;
use haru::vmbindings::value::Value;
use haru::vmbindings::vm::{Vm, VmOpcode};

macro_rules! eval {
    ($x:expr) => {{
        let prog = grammar::start($x).unwrap();
        let mut c = compiler::Compiler::new();
        for stmt in prog {
            stmt.emit(&mut c);
        }
        c.cpushop(VmOpcode::OP_HALT);
        let mut vm = c.into_vm();
        hanayo::init(&mut vm);
        vm.gc_enable();
        vm.execute();
        vm
    }};
}

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            eval!(s);
        }
    });
}