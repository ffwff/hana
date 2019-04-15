# hana

**hana**, a small object oriented programming language.

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

## Fibonacci numbers

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
