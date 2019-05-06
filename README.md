# 🌸 hana

**hana**, a small object oriented programming language.

**NOTE:** hana's interpreter is under a rewrite in Rust! Please checkout the [master branch](https://github.com/ffwff/hana/tree/master)

## Cloning

You'll have to clone it recursively:

```
git checkout master
git clone --recursive https://github.com/ffwff/hana
```

## Building

(building was tested by using gcc-7 on an x64 with Linux, mileage may vary on other architectures)

You'll need to install **libffi** and **libgc** (BoehmGC garbage collector).

For release builds, just do:

```
make RELEASE=1
```

It is recommended that you build the interpreter with libreadline and the `ENABLE_READLINE` flag set for a better REPL.

To bootstrap the init bytecode, compile it using a debug/release build, then remake the interpreter:

```
make
make build/init.bin
make RELEASE=1 INCLUDE_BYTECODE=1
```

For debug:

```
make DEBUG=1
```

## Running

Once built, you can write hana code into a source file, then invoke the interpreter like this:

```
./main program.hana
```

Alternatively you could try things out in the REPL:

```
./main
```

In the REPL, to type a newline, simply put `\` at the end of the line then press enter.

## Documentation

*see [DOCUMENTATION.md](/DOCUMENTATION.md)*

## Examples

*see [DOCUMENTATION.md#Examples](/DOCUMENTATION.md#examples) for more*

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

## License

GPLv3 License
