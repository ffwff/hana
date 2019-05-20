# 🌸 hana

[![travis](https://travis-ci.org/ffwff/hana.svg?branch=haru)](https://travis-ci.org/ffwff/hana)
[![codecov](https://codecov.io/gh/ffwff/hana/branch/haru/graph/badge.svg)](https://codecov.io/gh/ffwff/hana)

**hana** is a small dynamically-typed scripting language written in Rust/C
and is inspired by Pascal, Ruby and Javascript. It primarily supports prototype-based
object orientation, dynamic arrays, first-class functions (with closure support). The interpreter
comes useful features such as a simple mark-and-sweep garbage collector, exception handling
and an import system.

**haru**, the Rust parser/runtime generates bytecode that runs on an optimised
virtual machine written in C (about as fast as Python and Ruby!)

## Installation

You'll need to have the cargo package manager and rust installed. You can then do:

```
cargo install haru
```

The interpreter called `haru` will be installed into your PATH.

## Running

Once built or installed, you can write hana code into a source file, then invoke the interpreter like this:

```
haru program.hana
```

Alternatively you could invoke a REPL for easier prototyping:

```
haru
```

For usage, pass the `-h` command:

```
usage: haru [options] [-c cmd | file | -]
options:
 -c cmd : execute program passed in as string
 -d/--dump-vmcode: dumps vm bytecode to stdout
                   (only works in interpreter mode)
 -b/--bytecode: runs file as bytecode
 -a/--print-ast: prints ast and without run
 -v/--version: version
```

## Examples

*see [/examples](https://github.com/ffwff/hana/tree/haru/examples) for more*

### Hello World

```
print("Hello World\n")
```

### Variables

```
name = "Alice"
age = 20
print(name, " is ", age, " years old.\n")
```

### Fibonacci numbers

```
// Regular recursive
fib(n) = n <= 1 ? 1 : fib(n-1) + fib(n-2)
print(fib(30), "\n")

// Faster recursive (with tail-call optimization!)
fibrec(n, prev, curr) = n <= 0 ? curr : fibrec(n-1, prev+curr, prev)
fib(n) = fibrec(n+1, 1, 0)
print(fib(50), "\n")
```

## Documentation

*see [DOCUMENTATION.md](https://github.com/ffwff/hana/blob/haru/DOCUMENTATION.md)*

## Building

(building was tested by using rust-nightly and gcc-4.8 on an x64 with Linux, mileage
may vary on other architectures)

Just do:

```
cargo build --release
```

## License

GPLv3 License
