Welcome to *hana*! Hana is a dynamically-typed interpreted language, built upon minimalist
philosophy, it's inspired by Pascal, Ruby and Javascript, and it's fairly fast and lightweight.
Enjoy your stay in Hana!

**This is alpha software! Things may break at any moment, please send bug report and stack trace to Issues!**

# Syntax

## Statements

All statements are separated by newlines. For statements that are composed
of a statement (X) which is preceded by `then`, if the statement X is a block
statement, the `then` keyword can be omitted, for example you can write:

```
if 1+1 == 2 begin
    print("math is correct")
end
```

Instead of

```
if 1+1 == 2 then begin
    print("math is correct")
end
```

### Expression

Syntax:

`[expression]`

Evaluates `[expression]`.

### If

Syntax (`else` line is optional):

```
if [expression] then [statement]
else [else statement]
```

The statement evaluates `[statement]` if `[expression]` is true, else
if `[else statement]` is specified then it will be evaluated.

### For

Syntax:

```
for [var]=[from] [to/downto] [dest] step [step] then [statement]
```

The statement sets `[var]` to `[from]`, and loops to (or down to) `[dest]` (inclusive)
stepping `[step]` each iteration. With each iteration, it executes `[statement]`.

`step [step]` is optional, if not specified, and the `to` keyword is used, `[step]` will be 1,
otherwise if `downto` is used, then `[step]` will be -1.

### For-in

Syntax:

```
for [var] in [object] then [statement]
```

The statement evaluates `[object]`.

* If `[object]` is an array, it will continuously set the variable `[var]` to every value
in the array and evaluate `[statement]`.
* If `[object]` is a string, it will continuously set the variable `[var]` to every character
in the string and evaluate `[statement]`.
* If `[object]` is an iterator, it will continuously evaluate the record's `next` function
until the record has a `stopped` key set.
* Otherwise, the interpreter will panic.

An iterator is simply a record with a `next` function, and is not initialized with a `stopped`
key.

### While

Syntax:

```
while [expression] then [statement]
```

The statement evaluates `[expression]`, if it true then continuously evaluate `[statement]`
until `[expression]` is false.

### Try statements

Syntax:

```
try
    [body]
case [type] as [e]
    [catch]
    ...
case [type2]
    ...
end
```

The statement tries to execute statements in `[body]`, if an exception is raised, it will
lookup the raised value's type in the `case` statements, once it finds a match, it will
execute the statements after the matching case statement.

Use `as [e]` in a case statement to capture the raised value into the variable `[e]`.

#### Raise statement

Syntax:

```
raise [value]
```

### Blocks

Syntax:

```
begin
[statements]
end
```

The statement evaluates all of `[statements]` sequentially.

### Function

(see [Types#Functions](#functions))

#### Scoping rules

A scope is a container which stores local variables. Every time a function is called,
a new scope is setup. Scopes can be nested, meaning functions (A) inside of functions (B) can
get its parent's variables (B), but not the other way around. Scopes can't set its parent
scope's variables, doing so will **create a new local variable with the same name**.

Example:

```
function a() begin
    x = 1
    function b() begin
        y = 2
        print(y," ", x,"\n") // => 2 1
        x = 3
        print(x,"\n") // => 3
    end
    print(x,"\n") // => 1
    b()
    print(x,"\n") // => 1
end

a()
```

Local variables inside of scopes can be used inside of functions that escapes the scope
(through returns, global variables or through dictionary/array keys). These are called
[closures](https://en.wikipedia.org/wiki/Closure_(computer_programming)). Hana implements
this by copying the current scope's local variables (whenever the function is declared)
into the function's hidden variable container.

```
function adder(n) begin
    return function(x) begin
        return x + n
    end
end

x = adder(5)
y = x(10) // => 15
```

Functions can do recursion, calling itself in its scopes:

```
function call_1000_times(n) begin
    print(n, "\n")
    if n == 1000 return
    return call_1000_times(n+1)
end
call_1000_times(0)
```

#### Return statements

The return statement exits the called function, and returns its result. If no result is given
or no return statement is specified in the function, the function will yield `nil`.

Syntax:

```
return [expr]
```

Return statements are only possible in function bodies.

### Record

(see [Types#Records](#records))

Record bodies only accept function, assignment and record statements.

### Use

(see [#Imports](#Imports))

## Expressions

The following sections describe expressions in hana in order of operator precedence.

### Assignment operators

Assignment operators assign the value of its right operand to its left operand.
The left operand may be a function call (for function definitions), a variable
or a member expression.

List of assignment operators:

| Shorthand operator | Meaning                        |
|--------------------|--------------------------------|
| `x = 10`           | Sets variable `x` to `10`.     |
| `x += 10`          | Sets variable `x` to `x + 10`. |
| `x -= 10`          | Sets variable `x` to `x - 10`. |
| `x *= 10`          | Sets variable `x` to `x * 10`. |
| `x /= 10`          | Sets variable `x` to `x / 10`. |

### Conditional operator

Hana uses the ternary operator to denote condition expressions:

```
condition ? then : otherwise
```

### Logical operators

`and`, `or`: Casts the two values into booleans then does the equivalent **short-circuited**
boolean operation.

`not` casts the value into a boolean then does a boolean NOT (`true` becomes `false` and
vice-versa).

### Equality operators

Compares 2 values to see if they are the same (`==`) or not the same (`!=`).

Integer-integer and float-float pairs will be compared value-wise.

If an integer is compared with a float, the float will be compared to a float-casted
value of the integer.

If a string is compared with another string, the byte values of the two strings will be
compared.

If an array is compared with another array, or a record is compared with another record,
they will **only be the same** when they are of the same memory address.

Otherwise, the two values will not be the same.

### Comparison operators

Compares 2 values to see if they are greater than (`>`), greater than or equal to (`>=`),
lesser than (`<`), lesser than or equal to (`<=`).

Numeric values will be compared value-wise.

String values will be compared based on their Unicode character points.

Otherwise comparison will always evaluate to `false`.

### Arithmetic operators

#### Division (/)

Divides right value from left value.

Type table:

| Left value | Right value | Result                 |
|------------|-------------|------------------------|
| Integer    | Integer     | Float                  |
| Integer    | Float       | Float                  |
| Float      | Integer     | Float                  |

#### Multiplication (*)

Multiplies 2 values together.

Type table:

| Left value | Right value | Result                 |
|------------|-------------|------------------------|
| Integer    | Integer     | Integer                |
| Integer    | Float       | Float                  |
| Float      | Integer     | Float                  |
| String     | Integer     | String (repeats string  by n times) |
| Array      | Integer     | Array (repeats array by n times) |

#### Modulo (mod)

Gets the modulo or the remainder when the left value is divided by the right value.

| Left value | Right value | Result                 |
|------------|-------------|------------------------|
| Integer    | Integer     | Integer                |

#### Addition (+)

Adds 2 values together.

Type table:

| Left value | Right value | Result                 |
|------------|-------------|------------------------|
| Integer    | Integer     | Integer                |
| Integer    | Float       | Float                  |
| Float      | Integer     | Float                  |
| String     | String      | String (concatenation) |

#### Subtraction (-)

Subtracts right value from left value.

Type table:

| Left value | Right value | Result                 |
|------------|-------------|------------------------|
| Integer    | Integer     | Integer                |
| Integer    | Float       | Float                  |
| Float      | Integer     | Float                  |

### Call expressions

Call expressions are used to call functions or records:

```
a() // calls function a with no arguments
a(1,2) // calls function a with 2 arguments
```

When a record is called, its `constructor` function is invoked, and depending on
the member expression's operator it will also pass a new dictionary in:

```
record Namespace
    record Example
        function constructor(self, num) begin
            self.num = num
            return self
        end
    end
end

Namespace.Example(10) // => calls constructor with arguments
                      // containing a new record and 10
Namespace::Example() // => calls constructor with arguments
                     // containing the Namespace record
```

### Member expressions

Member expressions are used to access a key of a record, an array or a string:

```
a.b // => access "b" key of record "a"
a["b"] // => access "b" key of record "a"
v = [3,5]
v[0] //  => access 0th index (value 3) of array "v"
s = "abcd"
s[3] // => "d" access 3rd element of string s
```

Calling a member expression will pass the left hand side as an argument to the
called method, to call a value's method without passing itself, use the `::` operator:

```
record Test
    function constructor(self) begin
        print(self)
    end
end
Test::constructor("Hello") // => Hello
```

## Values

Basic types include:

 * `String`: UTF-8 encoded string
 * `Int`: 64-bit signed integer
 * `Float`: 64-bit double precision floating point
 * `Function`: function
 * `Record`: records
 * `Array`: array

`Int`, `Float` values are primitives, they are passed by copies
into (arguments) and out of (return) functions.

`Function`, `String`, `Record` and `Array` values are passed by reference. Those values are only deleted
whenever the last variable containing the value is deleted or set to another value.

Primitive types have an immutable `prototype` key which provides its methods:
`String` for string literals, `Int` for integers, `Float` for floating points,
`Array` for arrays. Records by default do not have any `prototype` key.

By convention, type names are title-cased.

### Identifiers

All strings of alphabetical characters, followed by alphanumeric characters
or the characters `$`, `_`, `?`, `!` are considered an identifier.

Identifiers starting with `$` denotes an identifier in the global scope:

```
a = 1
f() = $a = 20
f() // sets the global variable a to 20
a // => 20
```

Otherwise identifiers will follow scoping rules.

Note that the `$` identifier will just refer to the `$` identifier.

### Functions

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

or use the expression form:

```
price = function(X) begin
    return X*0.5
end
```

or the Ruby style closure form (you should only use this for callbacks!):

```
price = |X| { return X*0.5 }
```

You can then call it like this:

```
fib(30) // => 1346269
```

Each function creates a scope (a separate variable environment). Variables that don't
exist in the upper scope will be declared locally, while existing variables will
set accordingly. (see [#Scoping rules](#scoping-rules))

If a return statement is issued within the function, and the expression within
the return statement is a call, then the interpreter will tail-call optimise it
by pushing the call expression's arguments onto the stack frame and jumping into
the function.

### Strings

Strings are mutable UTF-8 strings. Characters in hana are Unicode grapheme clusters
(actual characters) rather than bytes.

```
name = "Alice"
greeting = "Hello " + name // => "Hello Alice"
kaos_says = "ab"*3 + "a" // => "abababa"
multiline = "

    multi
    line
    string

"
```

Strings can be indexed using the brackets (`[]`) operator, however it is a O(n) operation
so you shouldn't do it too much! (store its characters into an array and access it)

Methods:

```
"abc".bytesize() // => 3 (number of bytes in string)
"abc".length() // => 3 (number of characters in string)
"abc".chars() // => ["a", "b", "c"] (characters in string into array)
"abc".delete(1,2) // => "a" (copies and deletes 2 characters starting from index 1)
"abc".delete!(1,2) // => "a" (deletes in place 2 characters starting from index 1)
"abc".copy(1, 2) // => "bc" (copies 2 characters starting from index 1)
"abc".index("bc") // => 1 (index of the string "bc" in "abc")
"abc".insert!(1, "bc") // => "abcbc" (inserts in place "bc" to "abc" in index 1)
"a b c".split(" ") // => ["a", "b", "c"] (splits string with delimeter ' ' into array)
"a".ord() // => 97 (turn first character of string into a number)
```

### Numbers

Numbers are either 64-bit ints or 64-bit double precision floats.

Numbers can be decimal or hexadecimal (prefixed by `0x`).

Methods:

```
97.chr() // => (int only) converts 97 to equivalent unicode character ('a')
```

### Records

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

Each time you call a record, the record's `constructor` function gets called instead,
and will pass the record in addition to your arguments. I recommend using `self` to denote
the argument containing the record. **You must return the record (self) manually in the constructor.**

Accessing a key can be done through member expressions. Accessing a non-existant key will throw an error.

To delete a record's key, simply set the key to `nil`.

Methods:

```
a = Record() // => creates a new record
a["key"] = "value" // => sets a key
Record::keys(a) // => ["key"] (gets all keys in record)
```

### Arrays

Arrays in Hana are dynamic arrays. You can declare arrays like this:

```
[] // => empty array
[1,2] // => array with elements 1, 2
array(1, 2) // => same
[1,2,3]*3 // => [1, 2, 3, 1, 2, 3, 1, 2, 3] (repeats all elements in [1,2,3] 3 times)
```

Methods:

```
[1,2,3].length() // => 3 (number of elements in array)
[].empty?() // => true (is array empty?)
[1,2,3].delete!(1,2) // => [1] (deletes 2 elements starting from index 1)
[1,2,3].copy(1, 2) // => [2,3] (copies 2 elements starting from index 1)
[1,2,3].index(2) // => 1 (index of the element 1 in array)
[1,2,3].insert(1, 1) // => [1,1,2,3] (inserts 1 to [1,2,3] in index 1)
a = [4,6,7,3,1]
a.sort() // => [1, 3, 4, 6, 7] (sorts array without changing the array)
a.sort!() // => [1, 3, 4, 6, 7] (sorts array in place)
a.map(f(x) = x*2) // => [8, 12, 14, 6, 2] (maps each element with result of function)
a.filter(f(x) = x > 5) // => [6,7] (filters based on the truthiness of the function)
a.reduce(f(prev, curr) = prev+curr, 0) // => 21
// reduce executes a function on each member of the array's element
// then returns a single output
```

## Comments

Comments can either be single line:

```
// a single line comment
```

or multiple lines:

```
/*
 multline comment
*/
```

# Libraries

## Standard library

Hana's standard library is called `hanayo`! The library is imported by default upon running.

### Special constants

```
nil // => (nil)
true // => 1
false // => 0
inf // => infinity
nan // => not a number
```

### Types

(see [#Types](#types))

### IO

```
print("Hello World") // => prints string "Hello World" onto stdout
input() // => gets a string from stdin
```

#### Files

```
f = File("/tmp/a", "r") // => opens the file /tmp/a with the read flag
f.read_up_to(10) // => read first 10 bytes of file as string
f.read() // => read all of file as string

f = File("/tmp/a", "w") // => opens the file /tmp/a with the writer flag
f.write("Hello World\n") // => overwrites the file with the string "Hello World\n"
```

### Environment variables

```
Env::get("x") // gets the environment variable x as a string
Env::set("x", "y") // sets the environment variable x to y
Env::vars() // gets a record of environment variables
```

### Commands

```
c = Cmd("echo Hello World") // creates a process with the shell command "echo Hello World"
c = Cmd(["echo", "Hello World"]) // creates a process with args "echo", "Hello World"
c.out() // => "Hello World\n" (spawns the process and gets its stdout)
c.err() // => "" (spawns the process and gets its stderr)
c.outputs() // => ["Hello World\n", ""] (spawns the process and gets an array of stdout/stderr outputs)
Cmd("cat -").in("hi").out() // => "" (spawns the process with "Hi" passed to its stdin and gets its stdout)
c.spawn() // => Process record (spawns and detaches the process)
```

### Process

```
p = Cmd(["echo", "Hello World"]).spawn()
p.out() // => "Hello World\n" (waits for the process to terminate and gets its stdout)
p.err() // => "" (waits for the process to terminate and gets its stderr)
p.outputs() // => ["Hello World\n", ""] (spawns the process and gets an array of stdout/stderr outputs)
p.in('hello') // => sends the string 'hello' into the process' stdin
```

## Optional libraries

In addition to the standard library, Hana also comes included with some optional libraries that
must be imported to use.

### JSON

```
JSON::parse('{"a": true}') // => record of a key = 1
(record
    a = JSON::true
end).to_json() // => {"a": true}
```

### C FFI

The implementation provides a native set of utilities for interfacing with C functions.

To enable it install the interpreter through cargo with the `cffi` feature. This
requires libffi be installed in your system.

```
// calling getuid(), a native libc function from the ffi
getuid = Cffi::Function("getuid", [], Cffi.Int64)
print(getuid.call([]), "\n")

// returning a unicode string from c ffi
strchr = Cffi::Function("strchr", [Cffi.String, Cffi.Int64], Cffi.String)
print(strchr.call(["Hello World", "r".ord()]), "\n")

// importing sin from libm and using it:
libm = Cffi::Library("/lib/libm.so.6")
sin = Cffi::Function(libm.sym("sin"), [Cffi.Float64], Cffi.Float64)
print(sin.call([0.4]), '\n')

// creating a garbage collected pointer
// free will automatically be called whenever
// it goes out of scope on garbage collection
malloc = Cffi::Function("malloc", [Cffi.UInt64], Cffi.Pointer)
free = Cffi::Function("free", [Cffi.UInt64], Cffi.Pointer)
print(Cffi::GcPointer(malloc.call([10]), free))
```

#### FFI Types

Stored in Cffi:

```
UInt8,
Int8,
UInt16,
Int16,
UInt32,
Int32,
UInt64,
Int64,
Float32,
Float64,
Pointer,
String,
Void
```

## Imports

You can import files using the `use` statement:

```
use [module]
```

Where `[module]` is a string,

* If the module name starts with a `./`, it will lookup and import the file relative to the script's local directory.
* If the module name starts with a `/`, it will lookup and import the file relative to the root filesystem directory.
* Otherwise, it will lookup and import the file in the path specified by `HANA_PATH` environment variable.

Circular dependency is undefined behavior. The interpreter may break if you do this.

# Virtual machine

Hana has a stack-based virtual machine written in C. Code passed on to the interpreter will
be parsed into an abstract syntax tree then translated into bytecodes which the virtual machine can
understand.

## Memory management

Hana uses garbage collection to manage memory. All values that are not referenced in the value stack or are not
referenced by any of the global objects will be automatically collected and free'd by the garbage collector.

For values on the native stack, the interpreter uses reference counting to track references. All heap-allocated values
that have a reference count of more than 0 will be considered a GC root (its childrens will not be marked as unreachable).
Otherwise the garbage collector will handle it as normal.

Upon exit the virtual machine will release all memory that is managed by the garbage collector.

## Native functions

Native functions callable from the virtual machine must have the following signature:

```c
void native_function(struct vm *vm, uint16_t nargs);
```

Arguments are passed on the virtual machine's stack, and the native
function is called with a pointer to the virtual machine and the number of
arguments (16-bits).

The native function must return a value by pushing it on the stack.
If you don't have anything to return, push nil to the stack.

You should be using the `hana_function` macro in the module `haru-decorator`
if you're exporting a Rust function.