# Regis

This is an interpreted programming language nobody will ever use.

This project is just an exercise in writing a programming language and the learning how the fiddly
bits that go along with it (lexer, parser, bytecode, etc.) work.

Regis is most similar to JavaScript, Python and Lua, with some influence from Rust in its syntax. Of course
it's actually currently far worse than any of these other languages, has no ecosystem, tooling,
editor support, learning materials or even a way to do most things a programming language needs to
do at the moment. It's awful. Don't use it.

But anyway, here's a hello world program:

```js
# examples/hello-world.regis

@println("Hello World!");
```

You can run it by running:

```
cargo run examples/hello-world.regis
```

There's also an implementation of Conway's Game of Life you can try:

```
cargo run examples/game-of-life.regis
```

Mind blowing, I know.

# Essentials

## Comments

Comments start with the "#" character and extend to the end of the line.

```js
# This is a comment.
```

## Built in Functions

Built in functions all start with "@" to avoid naming conflicts.

* *@print(value)* Print a value to stdout without a newline.
* *@println(value)* Print a value to stdout with a newline at the end.
* *@len(value)* Return the number of values in a list, pairs in an object or characters in a string.
* *@import(path)* Import exported variables and functions from another module as an object.

That's it for now. More will be added.

---

## Binary Operators

| Operator | Description |
| -------- | ----------- |
| +   | Add two numbers, concate two strings or lists, or merge two objects.
| \-  | Subtract two numbers.
| *   | Multiply two numbers.
| /   | Divide two numbers.
| <<  | Bit shift an integer left by some number of bits or append a value to a list.
| >>  | Bit shift an integer right by some number of bits.
| &   | Compute the bitwise "and" of two numbers.
| \|  | Compute the bitwise "or" of two numbers.
| and | Short circuiting "and" operation. Return the left value if it's false, otherwise compute and returns the right value.
| or  | Short circuiting "or" operation. Return the left value if it's true, otherwise compute and return the right value.
| ??  | Short circuiting "null coaless" operation. Return the left value if it's not null, otherwise compute and return the right value.
| <   | Return true if the number on the left is less than the number on the right, otherwise return false.
| >   | Return true if the number on the left is greater than the number on the right, otherwise return false.
| <=  | Return true if the number on the left is less than or equal to the number on the right, otherwise return false.
| >=  | Return true if the number on the left is greater than or equal to the number on the right, otherwise return false.
| ==  | Return true if the value on the left is equal to the value on the right, otherwise return false. Nulls, booleans, ints, floats and strings are compared by value. Lists and objects are equal if they are the same instance. Values of different types (other than comparisons of ints and floats) are not equal.
| !=  | Compute the inverse of "==".

---

## Unary Operators

| Operator | Description |
| -------- | ----------- |
| \-   | Negate the value of a number on the right.
| ~    | Compute the bitwise "not" of a number on the right.
| not  | Return false if the value on the right is truthy, otherwise return true.

## Chain Operators

| Operator | Description |
| -------- | ----------- |
| function(...args) | Call the function on the left with the specifed comma separated arguments.
| list-or-object[index] | Get the value at an index in a list or object.
| object.index | Shorthand for object["index"].

## Operator Precedence

* Expressions wrapped in parenthesis are computed first.
* All chain operators have higher precedence than unary operators.
* All chain operators have the same precedence and are evaluated left to right.
* All unary operators have higher precedence than binary operators.
* All unary operators have the same precedence and are evaluated right to left.
* Binary operators are evaluated in the following order, with higher precedence coming first:
  1. ??
  2. \* /
  3. \+ -
  4. &
  5. |
  6. << >>
  7. < > <= >=
  8. == !=
  9. and
  10. or
* Binary operators of the same precedence are evaluated left to right.

## Values

### Null

You know what it is. It's nothing.

```js
null
```

### Boolean

Either true or false.

```js
true
false
```

### Int

A 64-bit signed integer value.

```js
0
1
-1
200
-200
12345679
```

There is no hex notation yet. Sorry.

### Float

A 64-bit floating point value.

```js
0.0
1.0
1.1
-1.0
-1.1
200.5
-200.5
123456789.0
```

You can do math with a mix of integers and floats. You don't normally have to worry about the difference unless you're messing with bits or doing equality checks.

### String

An immutable UTF-8 encoded string of characters that represent text.

Strings can be indexed by character positions. In this language, strings and lists start at index 0. Indexing outside of a string or list will return null.

```js
""
"☺"
"abc"
@len("") # ==> 0
@len("abc") # ==> 3
"abc"[0] # ==> "a"
"abc"[1] # ==> "b"
"abc"[2] # ==> "c"
"abc"[3] # ==> null
"abc"[-1] # ==> null
```

Strings can be concatenated using the "+" operator.

```js
"ab" + "cd" # ==> "abcd"
```

Only double quoted strings are allowed.

### List

A mutable array that can contain an arbitrary number of values of any type.

```js
[]
[1, 2, 3]
[true, 1, "1", [1]]
@len([]) # ==> 0
@len([1, 2, 3]) # ==> 3
@len([1, 2, 3]) # ==> 3
[1, 2, 3][0] # ==> 1
[1, 2, 3][1] # ==> 2
[1, 2, 3][2] # ==> 3
[1, 2, 3][3] # ==> null
[1, 2, 3][-1] # ==> null
```

Values can be appended to a list using the "<<" operator.

```js
let numbers = [1, 2, 3];
numbers << 4;
numbers << 5;
numbers << 6 << 7 << 8;
@println(numbers); # [1, 2, 3, 4, 5, 6, 7, 8]
```

Lists can be concatenated using the "+" operator.

```js
[1, 2] + [3, 4] # ==> [1, 2, 3, 4]
```

Lists can have trailing commas.

```js
[1, 2, 3, 4,]
```

### Object

A mutable hashmap which can map arbitrary indices to arbitrary values. All values are valid indices. Equality of indices is the same as the "==" operator.

```js
{}
{ a: 1, b: 2, c: 3 }
{ "a": 1, "b": 2, "c": 3} # ==> { a: 1, b: 2, c: 3 }
{ ["a"]: 1, ["b"]: 2, ["c"]: 3 } # ==> { a: 1, b: 2, c: 3 }
```

```js
let object = { a: 1, b: 2, c: 3 };

@println(@len(object)); # 3

@println(object["a"]); # 1
@println(object["b"]); # 2
@println(object["c"]); # 3
@println(object["d"]); # null

@println(object.a); # 1
@println(object.b); # 2
@println(object.c); # 3
@println(object.d); # null

object["d"] = 4;
object.e = 5;

@println(object["d"]); # 4
@println(object["e"]); # 4

@println(object.d); # 4
@println(object.e); # 4

let numbers = { [100]: 1, [200]: 2, [300]: 3 };

@println(object[100]); # 1
@println(object[400]); # null
```

Objects can be merged using the "+" operator. If an index is common to both objects, the values of the right object will override those on the left.

```js
{ a: 1, b: 2 } + { b: 3, c: 4 } # ==> { a: 1, b: 3, c: 4 }
```

Objects can have trailing commas.
```js
{ a: 1, b: 2, c: 3, }
```

Function

I'm just going to assume we generally know what a function is.

Functions in this language are declared as follows:

```js
fn add(a, b) {
    return a + b;
}
```

They can also be declared with an expression body.

```js
fn sub(a, b) => a - b;
```

Functions are normal values you can pass around.

```js
let add = fn add(a, b) {
    return a + b;
};

let sub = fn sub(a, b) => a + b;

@println(sub(add(1, 2), 3)); # 0

let functions = [add, sub];
@println(functions[0](2, 2)); # 4
@println(functions[1](2, 2)); # 0
```

Functions can be anonymous.

```js
fn(a, b) {
    return a + b;
}

fn(a, b) => a - b
```
Parameter parenthesis can be omitted if the function is anonymous and has no parameters.

```js
fn {
    return 1
}

fn => 1;
```

Functions can capture and modify variables outside of their scope. This works pretty much the same as it does in JavaScript.

```js
let number = 0;

fn increment() {
    number += 1;
}

increment();
increment();
increment();

@println(number); # 3
```

Functions that don't specify a return value return null by default.

```js
fn nothing() { }

@println(nothing()); # null
```

## Truthyness

The following values are considered "falsey":

1. null
2. false
3. 0
4. 0.0

All other values are considered "truthy".
## Control Flow

### If Statement

If statements work as you would probably expect. If a specified condition evaluates to a "truthy" value, a block will be run.

```js
let condition = true;

# This will print "yes".
if condition {
    @println("yes");
} else {
    @println("no");
}
```

```js
let number = 0;

# This will print "bear".
if number == 0 {
    @println("lion");
} else if number == 1 {
    @println("bear");
} else {
    @println("wolf");
}
```

Braces are always required for if statements.

If statements aren't expressions... yet. They should be.

### While Loop

While loops are also exactly what you would expect.

```js
# This will print integers 0 to 99.

let i = 0;
while i < 100 {
    @println(i);
}
```

Braces are always required for while loops.

### For Loop

It doesn't exist yet. Iterators have to be implemented first.

## Modules

Variables and functions from other files can be imported via the built-in "@import" function.

```js
# math.regis

export PI = 3.14;

export fn add(a, b) {
    return a + b;
}
```

```js
# main.regis

let Math = @import("./math.regis");

@println(Math.PI); # 3.14
@println(Math.add(1, 2)); # 3
```

The "Math" variable in the above example is just an object containing the exported variables and functions of the "math.regis" module.

Modules are lazy loaded. Additional imports of the same module will return the same object.

## Future Work

There's a hell of a lot missing before I would consider this language "complete":

* Iterators
* For loops
* Block expressions
* Pipeline operator
* List and object destructuring
* Support for object oriented programming
* Math module
* String module
* List module
* Object module
* IO module
* Error values (Expected error handling)
* Exceptions (Unexpected error handling)
* REPL
* Reasonable interoperability with Rust as an embedded language
