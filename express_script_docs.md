# Express Script Language Documentation

Welcome to the documentation for **Express Script**, the custom programming language for Eclipse OS.

If you would like more addition feel free to ask!

---

## Table of Contents

- [Introduction](#introduction)
- [Syntax Overview](#syntax-overview)
- [Variables](#variables)
- [Operators](#operators)
- [Control Flow](#control-flow)
  - [If / Else](#if--else)
  - [While Loop](#while-loop)
  - [For Loop](#for-loop)
- [Functions](#functions)
- [I/O Operations](#io-operations)
- [Special Keywords](#special-keywords)
- [Examples](#examples)

---

## Introduction

Express Script is a lightweight, C-like scripting language designed for use in Eclipse OS. It supports variables, arithmetic, control flow, functions, and basic I/O.

---

## Syntax Overview

- **Statements** end with a semicolon (`;`).
- **Blocks** are enclosed in `{ ... }`.
- **Comments** start with `//` and continue to the end of the line.

---

## Variables

Declare a variable with `let`:

```js
let x = 10;
let name = "Eclipse";
```

Assign a new value:

```js
x = x + 5;
```

---

## Operators

| Operator | Description        | Example      |
|----------|--------------------|--------------|
| `+`      | Addition           | `a + b`      |
| `-`      | Subtraction        | `a - b`      |
| `*`      | Multiplication     | `a * b`      |
| `/`      | Division           | `a / b`      |
| `^`      | Power              | `a ^ b`      |
| `++`     | Increment          | `x++`        |
| `--`     | Decrement          | `x--`        |
| `==`     | Equal              | `a == b`     |
| `!=`     | Not equal          | `a != b`     |
| `<`      | Less than          | `a < b`      |
| `<=`     | Less or equal      | `a <= b`     |
| `>`      | Greater than       | `a > b`      |
| `>=`     | Greater or equal   | `a >= b`     |
| `&&`     | Logical AND        | `a && b`     |
| `||`     | Logical OR         | `a || b`     |
| `!`      | Logical NOT        | `!a`         |

---

## Control Flow

### If / Else

```js
if (x > 0) {
    print("Positive");
} else {
    print("Non-positive");
}
```

### While Loop

```js
let i = 0;
while (i < 5) {
    println(i);
    i++;
}
```

### For Loop

```js
for (let i = 0; i < 5; i++) {
    println(i);
}
```

---

## Functions

Define a function with `fn`:

```js
fn add(a, b) {
    return a + b;
}

let result = add(2, 3);
println(result); // 5
```

---

## I/O Operations

- `print(value)` — Print without newline.
- `println(value)` — Print with newline.

```js
print("Hello, ");
println("world!");
```

---

## Special Keywords

- `let` — Declare a variable
- `fn` — Define a function
- `if`, `else` — Conditional statements
- `while` — Loop
- `for` — Loop
- `return` — Return from function
- `true`, `false` — Boolean values
- `asm` — Inline assembly (advanced)

---

## Examples

### Factorial Function

```js
fn fact(n) {
    let result = 1;
    while (n > 1) {
        result = result * n;
        n--;
    }
    return result;
}

println(fact(5)); // 120
```

### FizzBuzz

```js
for (let i = 1; i <= 15; i++) {
    if (i % 15 == 0) {
        println("FizzBuzz");
    } else if (i % 3 == 0) {
        println("Fizz");
    } else if (i % 5 == 0) {
        println("Buzz");
    } else {
        println(i);
    }
}
```

---

## Comments

```js
// This is a comment
let x = 10; // Inline comment
```

---

## Inline Assembly

You can use `asm("...")` for advanced operations (if supported):

```js
asm("nop");
```

---

## License

Express Script is part of [Eclipse OS](https://github.com/ghostedgaming/eclipse-os).

---
