#![feature(vec_remove_item)]
#![feature(alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![allow(dead_code)]

use std::io::{self, Read, Write};
#[macro_use] extern crate decorator;
extern crate ansi_term;
use ansi_term::Color as ac;
use rustyline::Editor;
use rustyline::error::ReadlineError;

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

// command/file
enum ProcessArg<'a> {
    Command(&'a str),
    File(&'a str),
}

fn process(arg: ProcessArg, flag: ParserFlag) {
    let mut c = compiler::Compiler::new();
    let s : String =
        match arg {
            ProcessArg::Command(cmd) => {
                c.files.push("[cmdline]".to_string());
                cmd.to_string()
            },
            ProcessArg::File("-") => {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).unwrap_or_else(|err| {
                    println!("error reading from stdin: {}", err);
                    std::process::exit(1);
                });
                c.files.push("[stdin]".to_string());
                s
            },
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
                c.modules_loaded.insert(std::path::Path::new(&filename).to_path_buf());
                c.files.push(filename.to_string());
                s
            }
        };
    let prog = ast::grammar::start(&s).unwrap_or_else(|err| {
        print_error(&s, err.line, err.column,
            err.line, err.column,
            "parser error:", &format!("expected {}", {
                let expected : Vec<String> = err.expected.iter().map(|x| x.to_string()).collect();
                expected.join(", ")
            }));
        std::process::exit(1);
    });

    // dump ast if asked
    if flag.print_ast {
        println!("{:?}", prog);
        return;
    }

    // emit bytecode
    for stmt in prog {
        stmt.emit(&mut c);
    }

    // dump bytecode if asked
    if flag.dump_bytecode {
        io::stdout().write(c.vm.code.as_bytes()).unwrap();
        return;
    }

    // execute!
    c.vm.compiler = Some(&mut c);
    c.sources.push(s);
    set_root(&mut c.vm);
    hanayo::init(&mut c.vm);
    c.vm.code.push(VmOpcode::OP_HALT);
    c.vm.execute();
    handle_error(&c);
}

fn handle_error(c: &compiler::Compiler) {
    if c.vm.error != VmError::ERROR_NO_ERROR {
        {
            let smap = c.lookup_smap(c.vm.ip as usize).unwrap();
            let src = &c.sources[smap.fileno];
            let (line, col) = ast::pos_to_line(&src, smap.file.0);
            let (line_end, col_end) = ast::pos_to_line(&src, smap.file.1);
            let message = format!("{} at {}:{}:{}", c.vm.error, c.files[smap.fileno], line, col);
            print_error(&src, line, col, line_end, col_end, "interpreter error:", &message);
        }
        if !c.vm.localenv.is_null() {
            eprintln!("{}", ac::Red.bold().paint("backtrace:"));
            let mut env = c.vm.localenv;
            while env != unsafe{ c.vm.localenv_bp.sub(1) } {
                let ip = unsafe{ &*env }.retip as usize;
                if let Some(smap) = c.lookup_smap(ip) {
                    let src = &c.sources[smap.fileno];
                    let (line, col) = ast::pos_to_line(&src, smap.file.0);
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
    }
}

// repl
fn repl(flag: ParserFlag) {
    let mut rl = Editor::<()>::new();
    let mut c = compiler::Compiler::new();
    c.files.push("[repl]".to_string());
    c.sources.push(String::new());
    c.vm.compiler = Some(&mut c);
    set_root(&mut c.vm);
    hanayo::init(&mut c.vm);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(s) => {
                rl.add_history_entry(s.as_str());
                match ast::grammar::start(&s) {
                    Ok(prog) => {
                        if flag.print_ast {
                            println!("{:?}", prog);
                            continue;
                        }
                        c.vm.error = VmError::ERROR_NO_ERROR;
                        c.sources[0] = s.clone();
                        c.vm.ip = c.vm.code.len() as u32;
                        for stmt in prog {
                            stmt.emit(&mut c);
                        }
                        c.vm.code.push(VmOpcode::OP_HALT);
                        c.vm.execute();
                        handle_error(&c);
                    }
                    Err(err) => {
                        print_error(&s, err.line, err.column,
                            err.line, err.column,
                            "parser error:", &format!("expected {}", {
                                let expected : Vec<String> = err.expected.iter().map(|x| x.to_string()).collect();
                                expected.join(", ")
                            }));
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                continue
            },
            Err(ReadlineError::Eof) => {
                println!("exitting...");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
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
 -v/--version: version", program)
}

fn version() {
println!(
"interpreter for the hana programming language (alpha).

This program is free software: you can redistribute it
and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.")
}

// parser flags
struct ParserFlag {
    pub dump_bytecode: bool,
    pub print_ast: bool,
}

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();

    let mut flags = ParserFlag{
        dump_bytecode: false,
        print_ast: false
    };
    let mut cmd = false;
    for arg in args {
        if arg != "-" && arg.starts_with('-') {
            match arg.as_str() {
            "-h" | "--help" => { return help(&program); },
            "-v" | "--version" => { return version(); },
            "-d" | "--dump-vmcode" => { flags.dump_bytecode = true; },
            "-a" | "--print-ast" => { flags.print_ast = true; },
            "-c" => { cmd = true; },
            _ => { println!("{}: invalid argument", program); return; }
            }
        } else if cmd {
            return process(ProcessArg::Command(&arg), flags);
        } else {
            return process(ProcessArg::File(&arg), flags);
        }
    }

    repl(flags)
}