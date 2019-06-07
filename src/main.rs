#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate num_derive;

cfg_if! {
    if #[cfg(jemalloc)] {
        extern crate jemallocator;
        #[global_allocator]
        static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
    }
}

use std::io::{self, Read, Write};
#[macro_use]
extern crate decorator;
extern crate ansi_term;
use ansi_term::Color as ac;
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod compiler;
#[macro_use]
mod ast;
mod vmbindings;
use vmbindings::vm::{Vm, VmOpcode};
use vmbindings::vmerror::VmError;
mod hanayo;

fn print_error(
    s: &String, lineno: usize, col: usize, _lineno_end: usize, col_end: usize, etype: &str,
    message: &String,
) {
    let line = s.split("\n").nth(lineno - 1).unwrap();
    let lineno_info = format!("{} | ", lineno);
    let lineno_info_len = lineno_info.len();
    eprintln!(
        "
{}{}
{}

{} {}",
        ac::Blue.bold().paint(lineno_info),
        line,
        ac::Blue.bold().paint(
            " ".repeat(lineno_info_len + col - 1)
                + &"^".repeat(if col_end > col { col_end - col } else { 1 })
        ),
        ac::Red.bold().paint(etype.to_string()),
        message
    );
}

// command/file
enum ProcessArg<'a> {
    Command(&'a str),
    File(&'a str),
}

fn process(arg: ProcessArg, flag: ParserFlag) {
    let mut c = compiler::Compiler::new();
    let s: String = match arg {
        ProcessArg::Command(cmd) => {
            c.modules_info
                .borrow_mut()
                .files
                .push("[cmdline]".to_string());
            cmd.to_string()
        }
        ProcessArg::File("-") => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s).unwrap_or_else(|err| {
                println!("error reading from stdin: {}", err);
                std::process::exit(1);
            });
            c.modules_info
                .borrow_mut()
                .files
                .push("[stdin]".to_string());
            s
        }
        ProcessArg::File(filename) => {
            let mut file = std::fs::File::open(&filename).unwrap_or_else(|err| {
                println!("error opening file: {}", err);
                std::process::exit(1);
            });
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap_or_else(|err| {
                println!("error reading file: {}", err);
                std::process::exit(1);
            });
            let mut modules_info = c.modules_info.borrow_mut();
            modules_info
                .modules_loaded
                .insert(std::path::Path::new(&filename).to_path_buf());
            modules_info.files.push(filename.to_string());
            s
        }
    };
    let prog = ast::grammar::start(&s).unwrap_or_else(|err| {
        print_error(
            &s,
            err.line,
            err.column,
            err.line,
            err.column,
            "parser error:",
            &format!("expected {}", {
                let expected: Vec<String> = err.expected.iter().map(|x| x.to_string()).collect();
                expected.join(", ")
            }),
        );
        std::process::exit(1);
    });

    // dump ast if asked
    if flag.print_ast {
        println!("{:?}", prog);
        return;
    }

    // emit bytecode
    for stmt in prog {
        if let Err(e) = stmt.emit(&mut c) {
            // TODO: better error message
            eprintln!("{:?}", e);
            return;
        }
    }
    c.cpushop(VmOpcode::OP_HALT);

    // dump bytecode if asked
    if flag.dump_bytecode {
        io::stdout().write(c.code_as_bytes()).unwrap();
        return;
    }

    // execute!
    c.modules_info.borrow_mut().sources.push(s);
    let mut vm = c.into_vm();
    hanayo::init(&mut vm);
    vm.gc_enable();
    vm.execute();
    handle_error(&vm, &c);
}

fn handle_error(vm: &Vm, c: &compiler::Compiler) -> bool {
    if vm.error != VmError::ERROR_NO_ERROR {
        if let Some(smap) = c.lookup_smap(vm.ip() as usize) {
            let src: &String = &c.modules_info.borrow().sources[smap.fileno];
            let (line, col) = ast::pos_to_line(&src, smap.file.0);
            let (line_end, col_end) = ast::pos_to_line(&src, smap.file.1);
            let message = format!(
                "{} at {}:{}:{}",
                vm.error,
                c.modules_info.borrow().files[smap.fileno],
                line,
                col
            );
            print_error(
                &src,
                line,
                col,
                line_end,
                col_end,
                "interpreter error:",
                &message,
            );
        } else {
            println!("interpreter error: {}", vm.error);
            return true;
        }
        if let Some(hint) = unsafe{ vm.error.hint(vm) } {
            eprintln!("{} {}", ac::Red.bold().paint("hint:"), hint);
        }
        let envs = vm.localenv_to_vec();
        if envs.len() > 0 {
            eprintln!("{}", ac::Red.bold().paint("backtrace:"));
            for env in envs {
                let ip = env.retip as usize;
                if let Some(smap) = c.lookup_smap(ip) {
                    let modules_info = c.modules_info.borrow();
                    let src = &modules_info.sources[smap.fileno];
                    let (line, col) = ast::pos_to_line(&src, smap.file.0);
                    eprintln!(
                        " from {}{}:{}:{}",
                        if let Some(sym) = modules_info.symbol.get(&ip) {
                            sym.clone() + "@"
                        } else {
                            "".to_string()
                        },
                        modules_info.files[smap.fileno],
                        line,
                        col
                    );
                } else {
                    eprintln!(" from bytecode index {}", ip);
                }
            }
        }
        true
    } else {
        false
    }
}

// repl
fn repl(flag: ParserFlag) {
    let mut rl = Editor::<()>::new();
    let mut c = compiler::Compiler::new();
    {
        let mut modules_info = c.modules_info.borrow_mut();
        modules_info.files.push("[repl]".to_string());
        modules_info.sources.push(String::new());
    }
    let mut vm = Vm::new(None, Some(c.modules_info.clone()));
    hanayo::init(&mut vm);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(s) => {
                rl.add_history_entry(s.as_str());
                c.modules_info.borrow_mut().sources[0] = s.clone();
                match ast::grammar::start(&s) {
                    Ok(mut prog) => {
                        if flag.print_ast {
                            println!("{:?}", prog);
                            continue;
                        }
                        let gencode = |c: &mut compiler::Compiler| -> Result<bool, ast::ast::CodeGenError> {
                            if let Some(_) = prog.last() {
                                let stmt = prog.pop().unwrap();
                                for stmt in prog {
                                    stmt.emit(c)?;
                                }
                                if let Some(expr_stmt) = stmt.as_any().downcast_ref::<ast::ast::ExprStatement>() {
                                    expr_stmt.expr.emit(c)?;
                                    return Ok(true);
                                } else {
                                    stmt.emit(c)?;
                                }
                            } else {
                                for stmt in prog {
                                    stmt.emit(c)?;
                                }
                            }
                            Ok(false)
                        };
                        // setup
                        #[allow(unused_assignments)]
                        let mut pop_print = false;
                        if vm.code.is_none() {
                            match gencode(&mut c) {
                                Ok(pop_print_) => {
                                    pop_print = pop_print_;
                                    c.cpushop(VmOpcode::OP_HALT);
                                    vm.code = Some(c.take_code());
                                    vm.execute();
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            }
                        } else {
                            vm.error = VmError::ERROR_NO_ERROR;
                            let len = vm.code.as_ref().unwrap().len() as u32;
                            c.receive_code(vm.code.take().unwrap());
                            match gencode(&mut c) {
                                Ok(pop_print_) => {
                                    pop_print = pop_print_;
                                    if c.clen() as u32 == len {
                                        continue;
                                    }
                                    c.cpushop(VmOpcode::OP_HALT);
                                    vm.code = Some(c.take_code());
                                    vm.jmp(len);
                                    vm.execute();
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            }
                        }
                        if !handle_error(&vm, &c) && pop_print {
                            println!("=> {:?}", unsafe{ vm.stack.pop().unwrap().unwrap() });
                        }
                    }
                    Err(err) => {
                        print_error(
                            &s,
                            err.line,
                            err.column,
                            err.line,
                            err.column,
                            "parser error:",
                            &format!("expected {}", {
                                let expected: Vec<String> =
                                    err.expected.iter().map(|x| x.to_string()).collect();
                                expected.join(", ")
                            }),
                        );
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                println!("exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

// CLI specific
fn help(program: &str) {
    println!(
        "usage: {} [options] [-c cmd | file | -]
options:
 -c cmd : execute program passed in as string
 -d/--dump-vmcode: dumps vm bytecode to stdout
                   (only works in interpreter mode)
 -b/--bytecode: runs file as bytecode
 -a/--print-ast: prints ast and without run
 -v/--version: version",
        program
    )
}

fn version() {
    println!(
        "haru: interpreter implemententation for the hana programming language.
version {}

This program is free software: you can redistribute it
and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.",
        env!("CARGO_PKG_VERSION")
    )
}

// parser flags
struct ParserFlag {
    pub dump_bytecode: bool,
    pub print_ast: bool,
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();

    let mut flags = ParserFlag {
        dump_bytecode: false,
        print_ast: false,
    };
    let mut cmd = false;
    for arg in args {
        if arg != "-" && arg.starts_with('-') {
            match arg.as_str() {
                "-h" | "--help" => {
                    return help(&program);
                }
                "-v" | "--version" => {
                    return version();
                }
                "-d" | "--dump-vmcode" => {
                    flags.dump_bytecode = true;
                }
                "-a" | "--print-ast" => {
                    flags.print_ast = true;
                }
                "-c" => {
                    cmd = true;
                }
                _ => {
                    println!("{}: invalid argument", program);
                    return;
                }
            }
        } else if cmd {
            return process(ProcessArg::Command(&arg), flags);
        } else {
            return process(ProcessArg::File(&arg), flags);
        }
    }

    repl(flags)
}
