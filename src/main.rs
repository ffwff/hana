#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![allow(dead_code)]

use std::io::Read;
#[macro_use] extern crate decorator;
extern crate ansi_term;
use ansi_term::Color as ac;

mod compiler;
#[macro_use] mod ast;
mod vmbindings;
use vmbindings::vm;
use vmbindings::vmerror::VmError;
use vmbindings::vm::VmOpcode;
use vmbindings::gc::set_root;
mod hanayo;

fn print_error(s: &String, lineno: usize, col: usize, _lineno_end: usize, col_end: usize,
               etype: &str, message: &String) {
    let line = s.split("\n").nth(lineno-1).unwrap();
    let lineno_info = format!("{} | ", lineno);
    let lineno_info_len = lineno_info.len();
    eprintln!("
{}{}
{}

{} {}",
    ac::Blue.bold().paint(lineno_info),
    line,
    ac::Blue.bold().paint(" ".repeat(lineno_info_len + col-1) +
        &"^".repeat(if col_end > col { col_end - col } else { 1 })),
    ac::Red.bold().paint(etype.to_string()),
    message);
}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    let mut file = std::fs::File::open(&args[1]).unwrap_or_else(|err| {
        println!("error opening file: {}", err);
        std::process::exit(1);
    });
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap_or_else(|err| {
        println!("error reading file: {}", err);
        std::process::exit(1);
    });
    let prog = ast::grammar::start(&s).unwrap_or_else(|err| {
        print_error(&s, err.line, err.column,
            err.line, err.column,
            "parser error:", &format!("expected {}", {
                let expected : Vec<String> = err.expected.iter().map(|x| x.to_string()).collect();
                expected.join(", ")
            }));
        std::process::exit(1);
    });
    let mut c = compiler::Compiler::new();
    c.files.push(args[1].clone());
    for stmt in prog {
        stmt.emit(&mut c);
    }
    set_root(&mut c.vm);
    hanayo::init(&mut c.vm);
    c.vm.code.push(VmOpcode::OP_HALT);
    c.vm.execute();

    if c.vm.error != VmError::ERROR_NO_ERROR {
        {
            let smap = c.lookup_smap(c.vm.ip as usize).unwrap();
            let (line, col) = ast::pos_to_line(&s, smap.file.0);
            let (line_end, col_end) = ast::pos_to_line(&s, smap.file.1);
            let message = format!("{} at {}:{}:{}", c.vm.error, c.files[smap.fileno], line, col);
            print_error(&s, line, col, line_end, col_end, "interpreter error:", &message);
        }
        if !c.vm.localenv.is_null() {
            eprintln!("{}", ac::Red.bold().paint("backtrace:"));
            let mut env = c.vm.localenv;
            while env != unsafe{ c.vm.localenv_bp.sub(1) } {
                let ip = unsafe{ &*env }.retip as usize;
                if let Some(smap) = c.lookup_smap(ip) {
                    let (line, col) = ast::pos_to_line(&s, smap.file.0);
                    eprintln!(" from {}{}:{}:{}",
                              if let Some(sym) = c.symbol.get(&ip) { sym.clone() + "@" }
                              else { "".to_string() },
                              c.files[smap.fileno], line, col);
                } else {
                    eprintln!(" from bytecode index {}", ip);
                }
                env = unsafe { env.sub(1) };
            }
        }
        std::process::exit(1);
    }
}