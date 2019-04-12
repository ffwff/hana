- dynamically-typed interpreted language (duck-typed?)
- similar to Pascal, inspired by Python, crystal

# Examples

## Hello World

```
print('Hello World')
```

## Variables

```
name = "Alice"
age = 20
```

## Functions

```
function fib(n) begin
    if n <= 1 return n
    return fib(n-1) + fib(n-2)
end
fib(30) // => 1346269
```

## Strings

```
name = "Bob"
greeting = "Hello " + name // => "Hello Bob"
blub = "ab"*3 + "a" // => "abababa"
```

## Objects

Prototype-based language like JS

Define structs like this:

```
record Person
    name = "Human"
    age = 0

    function constructor(name, age) begin
        this.name = name
        this.age = age
    end
end
```

Instance:

```
constructorless = record(Person) // => { name: "Human", age: 0 }
bob = record(Person, "Bob", 20)
// or
alice = Person("Alice", 20)
```

Use as dictionary:

```
bob = record() // empty record
bob.name = "Bob"
```

## Types

Basic types:

```
str, int, float, record, struct, function
```
