<div align="center">

# 🟩 GeLang 💥

**A statically typed language with ownership, borrowing, and short syntax.**

No OOP · No garbage collector · Full control over memory · Blazing fast

[Features](#-features) ·
[Quick Start](#-quick-start) ·
[Variables](#-variables) ·
[Ownership](#-ownership) ·
[Structs](#-structs)

---

</div>

---

## 🗺️ Roadmap

GeLang is built in four phases. Each phase makes the language more complete.

| Phase | Goal | Status |
|---|---|---|
| **1** | Core — Turing-complete | ✅ Done |
| **2** | Ergonomics | ❌ Not started |
| **3** | Safety | ❌ Not started |
| **4** | Types | ❌ Not started |

### ✅ Phase 1 — Core (Turing-complete)

The language can already compute anything. It compiles to C.

- [x] **Lexer** — all tokens from the spec
- [x] **Parser** — types, variables, `fn`, `if` / `elif` / `else`, `while`, `loop`, `return`
- [x] **Emitter** — GeLang → C

### ❌ Phase 2 — Ergonomics

Short syntax and better control flow.

- [ ] `for..in` and ranges (`0..10`)
- [x] `match` / `switch`
- [ ] `emit` — return a value from a loops/match/switch/if
- [ ] Named loops (`break <name>` / `continue <name>`)
- [x] `inline fn` and `pub fn`
- [x] Arrow functions (`fn add(...) i32 => a + b;`)

### ❌ Phase 3 — Safety

Memory rules checked by the compiler.

- [ ] Ownership (`move`, `copy`)
- [ ] Borrowing (`&`, `&mut`)
- [ ] Lifetimes
- [ ] Null safety (`i32?`, `??`, `let?`)

### ❌ Phase 4 — Types

User-defined types and containers.

- [ ] Structs + `impl`
- [ ] Enums
- [ ] `arr<T>`, `vec<T>`, `heap<T>`
- [ ] `char8/16/32`, `str8/16/32`

> 📌 **Note:** The sections below describe the full language design.
> Features from Phases 2–4 are part of the spec, but the compiler does not support them yet.

---

</div>

---

## 📑 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Variables](#-variables)
- [Types](#-types)
- [Operators](#-operators)
- [Functions](#-functions)
- [If / Else](#-if--else)
- [Match / Switch](#-match--switch)
- [Loops](#-loops)
- [Emit — Return a Value From a Loop](#-emit--return-a-value-from-a-loop)
- [Print](#-print)
- [Comments](#-comments)
- [Strings and Chars](#-strings-and-chars)
- [Ownership](#-ownership)
- [Borrowing](#-borrowing)
- [Lifetime](#-lifetime)
- [Null and Optional Types](#-null-and-optional-types)
- [Structs](#-structs)
- [Ranges](#-ranges)
- [Enums](#-enums)
- [Cheat Sheet](#-cheat-sheet)

---

## ✨ Features

| | |
|---|---|
| 🔒 **Ownership and borrowing** | Values move by default. Use `&` and `&mut` to borrow. The compiler checks lifetimes. |
| 🧊 **Immutable by default** | `let` cannot change. `mut` can change. |
| 🎯 **Type inference** | You can skip the type. The compiler takes it from the value. |
| 🧩 **Everything is an expression** | `if`, `match`, and `loop` can return a value. |
| 🚫 **Safe null** | Optional types `T?` and the `??` operator. No surprise null errors. |
| 🏷️ **Named loops** | Write `break outer;` or `continue outer;`. No labels needed. |
| 📤 **`emit`** | Send a value out of a loop body. |
| 🧱 **Structs without OOP** | Fields, methods, and `impl` blocks. Like C, but easier. |
| 🎨 **Two styles** | Block style `{}` or arrow style `=>`. Pick what you like. |

---

## 🚀 Quick Start

```GeLang
fn add(i32 a, i32 b) i32 => a + b;

fn main() {
    let x = 10;
    mut i32 y = 32;

    y += add(x, 8);

    println("x = {}, y = {}", x, y);

    let sign = if y > 0 => "positive";
               else     => "non-positive";

    println(sign);
}
```

---

## 📦 Variables

### How to declare

```GeLang
let x = 12;          // immutable, type inferred
i32 x = 12;          // immutable, type written
mut x = 12;          // mutable, type inferred
mut i32 x = 12;      // mutable, type written
```

### Declare now, set later

You can declare a variable first and give it a value later.
An immutable variable can get a value **only once**.

```GeLang
i32 x;      // declare
x = 10;     // first value
x = 42;     // ❌ error: x is immutable
```

```GeLang
mut i32 x;  // declare
x = 10;
x = 43;
x = 65;     // ✅ fine: x is mutable
```

### Shadowing

A new `let` destroys the old variable and makes a new one. The type can change.

```GeLang
let x = 10;
let x = "hello";   // the old x is destroyed
```

### Rules

> ⚠️ You cannot turn an immutable variable into a mutable one. Only the other way works.

```GeLang
mut x = 10;      // make it mutable
let y = &x;      // y is a reference to mut x
```

---

## 🔢 Types

### Primitive types

| Type | Meaning |
|---|---|
| `usize`, `isize` | Unsigned/Signed size. Depends on the register size. |
| `i8`, `i16`, `i32`, `i64` | Signed integers |
| `u8`, `u16`, `u32`, `u64` | Unsigned integers |
| `f32`, `f64`, `f128` | Floating point numbers |
| `bool` | A flag: true or false |
| `bits` | 8 bits; array of flags |
| `byte` | One byte. Same as `u8`. |
| `char8`, `char16`, `char32` | A character in UTF-8 / UTF-16 / UTF-32 |
| `str8`, `str16`, `str32` | A string in UTF-8 / UTF-16 / UTF-32 |

### Container types

| Type | Meaning | Similar to |
|---|---|---|
| `arr<type, size>` | Fixed-size array | `[]` in C/C++ |
| `vec<type>` | Dynamic array | `std::vector` in C++ |
| `heap<type>` | A value on the heap. Smart pointer. | `std::unique_ptr` in C++ |

---

## ➗ Operators

### Compare

```
==   !=   <   >   <=   >=
```

### Math

| Operator | Action |
|---|---|
| `+` `-` `*` `/` | Add, subtract, multiply, divide |
| `+=` `-=` `*=` `/=` | Do the math and assign |
| `++` `--` | Increment / decrement |

> 💡 Every compound operator follows the same rule: `x ^= 3` means `x = x ^ 3`.

### Logic and bits

| Operator | Action |
|---|---|
| `!` | NOT |
| `&&` | AND with short-circuit |
| `&` | Bitwise AND |
| `\|\|` | OR with short-circuit |
| `\|` | Bitwise OR |

### Special

| Operator | Action |
|---|---|
| `? :` | Ternary operator |
| `??` | Unwrap an optional value, or use a fallback |

### Note about `++` and `--`

You can use them inside an expression. But they always run **before** the expression is calculated:

```GeLang
y = x-- + 3;
// this is the same as:
// x = x - 1;
// y = x + 3;
```

---

## 🧮 Functions

### Basic form

```GeLang
fn add(i32 a, i32 b) i32 {
    a + b            // implicit return: no semicolon
}
```

### Void function

```GeLang
fn greet() {         // no return type = void
    println("hi");
}
```

### Arrow form

```GeLang
fn add(i32 a, i32 b) i32 => a + b;      // short form, returns a value
fn print_num(i32 x) => println(x);      // no return type
fn print_num(i32 x) i32 => println(x);  // ❌ error: println returns nothing
```

### Every path must return a value

```GeLang
fn calculate(i32 x) i32 {
    if x > 0 => x        // ❌ error: not every path returns an i32
}
```

```GeLang
fn calculate(i32 x) i32 {
    if x > 0 => return x;
    0                    // ✅ fallback path
}
```

### Modifiers

| Modifier | Meaning |
|---|---|
| *(none)* | Private function |
| `pub` | Public function |
| `inline` | Guaranteed inline |

```GeLang
inline fn greet() {
    print("hi from inline");   // guaranteed inline
}

pub fn greet() {
    print("hi public");        // public
}
```

### Return

```GeLang
return expr;   // explicit return
expr           // last expression in a block, no ';' = implicit return
```

---

## 🔀 If / Else

### Block form

```GeLang
if x > 0 {
    println("pos");
} elif x == 0 {
    println("zero");
} else if x == 1 {       // elif == else if
    println("one");
} else {
    println("neg");
}
```

### If as an expression

```GeLang
let x = if 42 < a {
    42
} else {
    141
};
```

All branches must return the **same type**:

```GeLang
let y = if 42 < a {
    42.1
} else {
    51
};                       // ❌ error: i32 and f64 are not the same
```

```GeLang
let x = if 42 < a {
    println(x);
} else {
    141
};                       // ❌ error: branches have different types
```

If there is no `else`, you must use `let?`:

```GeLang
let? x = if 42 < a {     // if the condition is false, x will be null
    42
};

let x = if 42 < a {      // ❌ error: expected else after the body
    42
};
```

### Arrow form

```GeLang
let x = if 42 < a => 52;

if 42 < a => println(a);

if 42 < a => println(a);
else      => println(a + 1);
```

> ⚠️ Inside a function, an arrow branch needs `return` and `;`:

```GeLang
fn calculate(i32 x) i32 {
    if x > 0 => x            // ❌ error: expected a semicolon
    else 0
}

fn calculate(i32 x) i32 {
    if x > 0 => return x;    // ✅ correct
    else 0
}
```

### Ternary operator

```GeLang
let a = 42 < a ? 32 : 34;    // like in C++
```

---

## 🎯 Match / Switch

`match` and `switch` are **the same token**. Use whichever you like.

```GeLang
match x {
    10 => {},
    42 => {},
    _  => {},        // _ is the default branch
    // instead of _, you can bind the value:
    y => {},     // y is an immutable reference
}
```

```GeLang
match x * y {
    10 => {},
    z => {}, // z is a mutable variable, because x * y is expression
}
```

```GeLang
switch x {           // exactly the same
    10 => {},
    42 => {},
    _  => {},
}
```

If `switch` does not return a value, `_` is not required:

```GeLang
switch x {
    12 => println("12"),
}
```

### Match as an expression

All branches must return the same type:

```GeLang
let x = switch a {
    14 => 14,
    42 => 42,
    52 => {
        52 + 12
    },
    _  => 0,
};
```

---

## 🔁 Loops

### Loop kinds

```GeLang
for num in nums {}       // walk over a collection
for i in 0..10 {}        // range
while cond {}
loop {}                  // infinite loop
```

### Named loops

Put a name after `:` at the end of the loop header.

```GeLang
for num in nums: for_nums {
    while num > 42 {
        break for_nums;
        // or: continue for_nums;
    }
}
```

Nested loops cannot share the same name:

```GeLang
for x in xs: outer {
    for y in ys: outer {   // ❌ error: same name
        break outer;
    }
}
```

### Break / Continue

```GeLang
break;              // leave the current loop
break <name>;       // leave a named loop
continue;           // next step of the current loop
continue <name>;    // next step of a named loop
```

---

## 📤 Emit — Early Return From a Loop or Match

The keyword `emit` sends a value out of a block early. The rest of the block is skipped.
You can use it in loops and in `match` branches.
All branches must return the same type.

### In a loop

```GeLang
let result = loop {
    if cond => emit 42;
};
```

### In a match branch

Use `emit` to leave a long branch early, before the last expression.

```GeLang
let y = match x {
    42 => 141,
    _ => {
        let t = 12;

        if t < 32 => emit 42;   // leaves the branch here, y becomes 42

        t                       // this line never runs
    },
};
```

> 💡 Without `emit`, the branch would return `t` (12).
> With `emit`, it returns `42` and stops right there.

---

## 🖨️ Print

```GeLang
println(x);              // a value
println("hello");        // a string literal
println("{}", x);        // formatted output
```

---

## 💬 Comments

```GeLang
// one-line comment

/* multi-line
   comment */
```

---

## 🔤 Strings and Chars

```GeLang
""   // string
''   // char
```

---

## 🔐 Ownership

### Move by default

```GeLang
let x = 42;      // x owns the value
let y = x;       // ownership moves to y, x is no longer usable
println(x);      // ❌ error: use of a moved value
```

### Explicit `move`

```GeLang
let x = 42;
let y = move x;  // guaranteed move
println(x);      // ❌ error: use of a moved value
```

### Explicit `copy`

```GeLang
let x = 42;
let y = copy x;  // copies the whole value
println(x);      // ✅ correct
```

> 📌 `copy` works directly only with primitives.
> For a struct, write a `.copy()` method. Then `copy x` calls `x.copy()` for you.

```GeLang
let x = A::new();
let y = copy x;   // calls x.copy()
```

---

## 🔗 Borrowing

| Form | Rule |
|---|---|
| `&i32 x` | Immutable reference. Many allowed at once. |
| `&mut i32 x` | Mutable reference. Only one at a time. Exclusive access. |

### Function parameters

```GeLang
fn print_num1(&i32 x) {      // always a reference
    println(x);
}

fn print_num2(i32 x) {       // always a copy
    x++;                     // you can change it: it is a local copy
    println(x);
}

fn print_num3(&mut i32 x) {  // always a mutable reference
    x++;                     // changes the original value
    println(x);
}
```

```GeLang
fn inc(&mut i32 x) {
    x++;                     // or x += 1
}

fn main() {
    mut i32 x = 10;

    print_num1(&x);          // immutable borrow
    inc(&mut x);             // mutable borrow
    print_num2(&x);          // ❌ error: expected a copy, not a reference
    print_num2(x);           // ✅ correct
}
```

### Reference variables

```GeLang
mut x = 10;        // value
&let y = &x;       // reference
```

You cannot take `&mut` from an immutable owner:

```GeLang
let x = 10;
&mut y = &x;       // ❌ error: the original is immutable
```

### Returning a reference

```GeLang
fn get_ref(&i32 x) &i32 {
    &let a = &x;
    &let b = &a;
    println(b);    // prints the value of x
    b              // returns a reference to x
}
```

---

## ⏳ Lifetime

> 🔒 **A borrow cannot live longer than its owner.**

```GeLang
fn bad() &i32 {
    i32 x = 42;
    &x             // ❌ error: returns a reference to a local variable
}
```

---

## 🕳️ Null and Optional Types

This works like Kotlin or C#, or like `std::optional` in C++.

### Declaration

```GeLang
let? x = null;     // ❌ error: the type cannot be inferred
let? x = 42;       // ✅ correct
i32? x = null;     // ✅ correct
let y = null;      // ❌ error
```

### Unwrap with match

```GeLang
i32? x = 32;

match x {
    _     => println("error!"),
    y => println(y),     // y has type &i32, not i32?
}
```

Binding with `y` works only for optional values:

```GeLang
i32 x = 42;

match x {
    y => println(y),     // ❌ error: x is not optional
}
```

Write conditions **inside** the branch body:

```GeLang
match x {
    0     => {},
    y => if y > 100 => {},   // ✅ correct
    _     => {},
}

match x {
    0     => {},
    y if y > 100 => {},      // ❌ error: guard syntax is not supported
    _     => {},
}
```

Matching on null:

```GeLang
let y = match x {
    null => {
        println("Error!");
        0
    },
    z => z,
};
```

### Optional if

One branch must be `null`:

```GeLang
let? x = if 23 > y => 42;    // the type comes from the value
         else null;

let? x = if 23 > y => 42;    // ❌ error: no null branch
```

### The `??` operator

```GeLang
i32? x = 32;
&let y = x ?? 0;   // use the value if it is not null, otherwise use 0
&let y = x ??;     // ❌ error: the form is only <value> ?? <otherwise>
```

### Null with the ternary operator

```GeLang
i32? x = null;
let y = !x ? 0 : 42;   // if null -> 0, else 42 (! here means NOT)
let y = x!;            // ❌ DOES NOT EXIST: unexpected token
```

### `??` vs `!x ? a : b`

| Expression | Behavior |
|---|---|
| `&let y = x ?? 0` | Unwraps the optional and stores a **reference** |
| `let y = !x ? 0 : 42` | A normal ternary operator: `0` if null, otherwise `42` |

---

## 🧱 Structs

Structs in GeLang work like C: **data plus functions, no OOP and no inheritance**.

### Fields

```GeLang
i32 a = 42;        // private
pub i32 b = 42;    // public
priv i32 c = 42;   // private (explicit)
stat i32 d = 42;   // public static
```

### Methods

`self` is added automatically to every non-static method.

```GeLang
fn get() i32 {                 // private by default
    self.x;                    // returns x
}

pub mut fn set(i32 new_x) {    // public
    self.x = new_x;
}

priv mut fn set_another(i32 new_x) {   // forced private
    self.x = new_x * 2;
}
```

### Static methods

Static methods are **always public** and have no `self`.

```GeLang
stat fn set(i32 new_x) {
    self.x = new_x;            // ❌ error: self does not exist in a stat fn
}

stat fn get_last_id() {
    42 + 423 * 4               // ✅ correct: no self
}
```

### Two ways to write methods

**1. Inside the struct** (like C/C++):

```GeLang
struct A {
    let a = 1;
    pub fn add() => a + 1;
}
```

**2. In separate `impl` blocks** (like Rust). You can write many blocks, in any file of the project:

```GeLang
struct A {
    let a = 1;
}

impl A {
    pub fn add() => a + 1;
}
```

### Adding `copy` support

```GeLang
impl A {
    pub fn copy() A {
        A {
            a: copy self.a,
        }
    }
}
```

### Accessing fields

You cannot move a field into a variable. You can only take a reference:

```GeLang
let x = A::new(42);

let b = move x.a;   // ❌ error
let b = x.a;        // ❌ error
&let b = &x.a;      // ✅ a reference to the field
```

---

## 📏 Ranges

The syntax is `0..10`.

```GeLang
if x in 0..10 => println("x between 0 and 10");
```

---

## 🎨 Enums

Enums support simple variants and variants with data.

```GeLang
enum Token {
    Plus,
    Minus,
    Mul,
    Div,
    Num {
        i32 value;
    },
    Str {
        str32 value;
    },
}
```

```GeLang
let x = Token::Plus;

let y = Token::Num {
    value: 42,
};
```

---

## 📋 Cheat Sheet

| Task | Syntax |
|---|---|
| Immutable variable | `let x = 1;` / `i32 x = 1;` |
| Mutable variable | `mut x = 1;` / `mut i32 x = 1;` |
| Reference | `&let y = &x;` |
| Mutable reference | `&mut y = &mut x;` |
| Move | `let y = move x;` |
| Copy | `let y = copy x;` |
| Optional value | `i32? x = null;` / `let? x = 42;` |
| Unwrap with fallback | `x ?? 0` |
| Short function | `fn f(i32 a) i32 => a + 1;` |
| Short condition | `if c => a; else => b;` |
| Named loop | `for i in 0..10: outer { break outer; }` |
| Return from a loop | `let r = loop { emit 42; };` |
| Public item | `pub` |
| Static item | `stat` |
| Inline function | `inline fn f() {}` |

---

<div align="center">

**GeLang** — simple, explicit, safe.

</div>
