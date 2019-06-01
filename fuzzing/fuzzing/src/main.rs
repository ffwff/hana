#[macro_use]
extern crate afl;
extern crate haru;

use haru::ast::grammar;
use haru::compiler;
use haru::hanayo;
use haru::vmbindings::value::Value;
use haru::vmbindings::vm::{Vm, VmOpcode};

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            if let Ok(prog) = grammar::start(s) {
                let mut c = compiler::Compiler::new();
                for stmt in prog {
                    stmt.emit(&mut c);
                }
                c.cpushop(VmOpcode::OP_HALT);
                let mut vm = c.into_vm();
                hanayo::init(&mut vm);
                vm.gc_enable();
                vm.execute();
            }
        }
    });
}