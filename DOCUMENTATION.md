Welcome to hana! Hana is a dynamically-typed interpreted language, built upon minimalist
philosophy, it's inspired by Pascal, Ruby and Javascript, and it's fairly fast and lightweight.
Enjoy your stay in Hana!

**This is alpha software! Things may break at any moment, please send bug report and stack trace to issues!**

# Examples

## Hello World

```
print("Hello World\n")
```

## Variables

```
name = "Alice"
age = 20
print(name, " is ", age, " years old.\n")
```

## Reversing a binary tree

```
record Node
    function constructor(self, left, right) begin
        self.left = left
        self.right = right
        return self
    end
    function print(self, indent) begin
        if indent for i=1 to indent print(" ")
        if self.left.prototype == Node begin
            print("[node]\n")
            self.left.print(indent+1)
        end
        else print(self.left, "\n")

        if indent for i=1 to indent print(" ")
        if self.right.prototype == Node begin
            print("[node]\n")
            self.right.print(indent+1)
        end
        else print(self.right, "\n")
    end
    function reverse(self) begin
        left = self.left
        self.left = self.right
        self.right = left
        if self.left.prototype == Node self.left.reverse()
        if self.right.prototype == Node self.right.reverse()
    end
end

print("before:\n")
a = Node(Node(Node("C", "D"), "B"), "A")
a.print(0)
print("\nafter:\n")
a.reverse()
a.print(0)
```

## Sieve of Erathosnes

```
function sieve(n) begin
    prime = [true]*(n+1)
    p = 2
    while p*p <= n begin
        if prime[p] for i=p*2 to n-p+1 step p begin
            prime[i] = false
        end
        p += 1
    end
    for p=2 to n if prime[p] print(p, " ")
end
sieve(10000)
print("\n")
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

# Expressions

## n-ary expressions

Binary operators include (comma separated):

```
+, -, *, /, mod, ==, !=, >, <, >=, <=,
and, or, =
```

Unary operators include:

```
not, -
```

## call expressions

```
a() // calls function a with no arguments
a(1,2) // calls function a with 2 arguments
```

## member expressions

Access a key of a record like this:

```
a.b // => access "b" key of record "a"
a["b"] // => access "b" key of record "a"
v = [3,5]
v[0] //  => access 0th index (3) of array "v"
```

# Statements

All statements are separated by newlines.

## Function

(see [Types#Functions](#functions))

## Record

(see [Types#Records](#records))

## If

Syntax (`else` line is optional):

```
if [expression] [statement]
else [statement]
```

## While

Syntax:

```
while [expression] [statement]
```

While `[expression]` is true, do `[statement]`

## For

Syntax:

```
for [var]=[from] [to/downto] [dest] step [step] [statement]
```

The statement sets `[var]` to `[from]`, and loops to (or down to) `[dest]` (inclusive)
stepping `[step]` (or `1`/`-1`) each iteration. With each iteration, it executes `[statement]`.

## Blocks

Syntax:

```
begin
[statements]
end
```

# Types

Basic types include:

 * `string`: string
 * `integer`: 64-bit int
 * `float`: 64-bit double precision floating point
 * `record`: records
 * `array`: array
 * `function`: function

`string`, `integer`, `float`, `function` values are primitives, they are passed by copies
into (arguments) and out of (return) functions.

`record` and `array` values are passed by reference. Those values are only deleted whenever
the last variable containing the value is deleted or set to another value.

## Functions

You can define a function like this:

```
function fib(n) begin
    if n <= 1 return n
    return fib(n-1) + fib(n-2)
end
function price(X) return X*0.5
```

or use a short hand form:

```
fib(n) = n <= 1 ? n : fib(n-1)+fib(n-2)
```

You can then call it like this:

```
fib(30) // => 1346269
```

Each function creates a scope (a separate variable environment). Variables that don't
exist in the upper scope will be declared locally, while existing variables will
set accordingly.

## Strings

Strings are natively ASCII character arrays. There's no support for Unicode yet!

```
name = "Alice"
greeting = "Hello " + name // => "Hello Alice"
kaos_says = "ab"*3 + "a" // => "abababa"
```

Methods:

```
"abc".bytesize() // => 3 (number of bytes in string)
"abc".length() // => 3 (number of characters in string)
"abc".delete(1,2) // => "a" (deletes 2 characters starting from index 1)
"abc".copy(1, 2) // => "bc" (copies 2 characters starting from index 1)
"abc".at(1) // => "b" (character at index 1)
"abc".index("bc") // => 1 (index of the string "bc" in "abc")
"abc".insert(1, "bc") // => "abcbc" (inserts "bc" to "abc" in index 1)
```

## Numbers

Numbers are either 64-bit ints or 64-bit double precision floats.

Methods:

```
1.1.round() // => 1
            // (float only, rounds number to int)
```

## Records

Hana is has a prototype-based object system, like JS. Every value is a record (a key-value
mapping), which keys can be accessed by using the dot (`.key`) operator or the brackets
(`["key"]`) operator. A record (say record A), can inherit another record (record B), by
setting its prototype key (`A.prototype`) to the parent record (`B`).

Once set, keys from record B can be used or called from record A. Record A's keys will of course
override record B's keys if they are set. **There's no distinction between records and instances of records.**

You can define a record through a statement like this:

```
record Person
    species = "Human"
    function constructor(self, name, age) begin
        self.name = name
        self.age = age
    end
    function talk(self) begin
        print("*", self.species, " noises*\n")
    end
end
```

Or through an expression:

```
Person = record
    species = "Human"
    // ...
end
```

Then use it by doing this:

```
bob = Person("Bob", 20) // => record with keys name="Bob" and age="20"
bob.name // => "Bob"
bob.species // => "Human"
bob.talk() // => *Human noises*
```

## Arrays

Arrays in Hana are dynamic arrays. You can declare arrays like this:

```
[] // => empty array
[1,2] // => array with elements 1, 2
array(1, 2) // => same
[1,2,3] // => [1, 2, 3, 1, 2, 3, 1, 2, 3]
```

Methods:

```
[1,2,3].length() // => 3 (number of elements in array)
[1,2,3].delete(1,2) // => [1] (deletes 2 elements starting from index 1)
[1,2,3].copy(1, 2) // => [2,3] (copies 2 elements starting from index 1)
[1,2,3].at(1) // => 2 (element at index 1)
[1,2,3].index(2) // => 1 (index of the element 1 in array)
[1,2,3].insert(1, 1) // => [1,1,2,3] (inserts 1 to [1,2,3] in index 1)
a = [4,6,7,3,1]
a.sort() // => [1, 3, 4, 6, 7] (sorts array without changing the array)
a.sort!() // => [1, 3, 4, 6, 7] (sorts array in place)
```

# Standard library

Hana's standard library is called `hanayo`! The library is imported by default upon running.

## IO

```
print(v) // => prints value "v" onto stdout
v = input() // => gets a string from stdin
```

## Types

(see [#Types](#types))
