# 🌸 hana

**hana**, a small object oriented programming language.

## Cloning

You'll have to clone it recursively:

```
git clone --recursive https://github.com/ffwff/hana
```

## Building

(building is tested with gcc-7 on Linux, mileage may vary on other architectures)

For release builds, just do:

```
make RELEASE=1 NOLOG=1
```

It is recommended that you build the interpreter with libreadline for a better REPL:

```
make RELEASE=1 NOLOG=1 READLINE=1
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
