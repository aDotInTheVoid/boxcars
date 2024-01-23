---
title: "BoC in Rust: Interim Report"
author: Alona Enraght-Moony
date: 2024-01-23
bibliography: cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
papersize: a4
geometry: margin=2cm
---

## Background

### Rust

Rust [@rust_book] is a programming languages originally developed by Mozilla Research,
and currently maintained by a large cross-org team.

Rust's most important feature (for our purposes) is its system of **Ownership & Borrowing**.

### Ownership & Borrowing: A very fast introduction.

The core idea of **ownership** is that each value has a unique owner, and that value is dropped when the
owner goes out of scope. To quote The Book [@rust_book]:

> - Each value in Rust has an owner.
> - There can only be one owner at a time.
> - When the owner goes out of scope, the value will be dropped.

However with only these rules, programming in rust would be extremely
uneconomic. If a function took a string as a parameter, it would have "take
ownership" of that value, so the calling function could no longer use it.
[^return_values]

Therefore, rust introduces the additional notion of **borrowing**. A value can be borrowed in
one of two ways: by a shared reference (spelt `&T`), or an exclusive reference (`&mut T`). These are
also sometimes referred to as a immutable or mutable reference respectively [@dtolnay_ref].

A value may have many shared references to it at a given time, but if it has any
exclusive reference to it that reference must be the only one. With a shared
reference, you can only read from the value. An exclusive reference is required
to mutate it. More succinctly, rust references are "Aliasable XOR mutable" [@boats_smaller].

#### Consequence of this: No iterator invalidation

(Note, these examples adapted from "Two Beautiful Rust Programs" [@matklad_beautiful], because otherwise
I'd be recreating them from memory)

https://matklad.github.io/2020/07/15/two-beautiful-programs.html

```rust
fn main() {
  let mut xs = vec![1, 2, 3];
  let x: &i32 = &xs[0];
  xs.push(92);
  println!("{}", *x);
}
```

```
error[E0502]: cannot borrow `xs` as mutable because it is also borrowed as immutable
 --> <source>:4:3
  |
3 |   let x: &i32 = &xs[0];
  |                  -- immutable borrow occurs here
4 |   xs.push(92);
  |   ^^^^^^^^^^^ mutable borrow occurs here
5 |   println!("{}", *x);
  |                  -- immutable borrow later used here

error: aborting due to previous error
```

whereas in C++

```cpp
#include <vector>
#include <iostream>

int main() {
    std::vector<int> xs {1, 2, 3};
    int* x = &xs[0];
    xs.push_back(4);
    std::cout << *x << '\n'; 
}
```

prints nonsence.

#### Consequence of this: Fearless concurrency

```rust
use std::thread::scope;
use std::sync::{Mutex, MutexGuard};

fn main() {
  let mut counter = Mutex::new(0);

  scope(|s| {
    for _ in 0..10 {
      s.spawn(|| {
        let mut guard: MutexGuard<i32> = counter.lock().unwrap();
        *guard += 1;
      });
    }
  });

  let total: &mut i32 = counter.get_mut().unwrap();
  *total += 1;
  println!("total = {total}");
}
```

1. Each thread only accesses the counter through the mutex.
2. Each thread can read from `main`'s stack.
3. Final increment doesn't need to lock mutex, no other threads could have it.

If any of these properties had changed, we'd get a compiler error.

### Ownership & Borrowing can't save you from everything

However, one area where Rust's type system doesn't do anything to help you is avoiding deadlocks.
One can trivially perform one by aquireing two mutex's in different orders:

```rust
use std::sync::Mutex;
use std::thread::{scope, sleep_ms};

fn main() {
    let m1 = Mutex::new(());
    let m2 = Mutex::new(());

    scope(|s| {
        s.spawn(|| {
            let g1 = m1.lock();
            sleep_ms(100);
            println!("t1: got m1, trying to get m2");
            let g2 = m2.lock();
            println!("t1: got both");
        });

        s.spawn(|| {
            let g2 = m2.lock();
            sleep_ms(100);
            println!("t2: got m2, trying to get m1");
            let g1 = m1.lock();
            println!("t2: got both");
        });
    });
}
```

While this example is obviously contrived, it demonstrates an important point.
Rust deadlocks have come up in practice, and are usually much more subtle than this
[@snoyman_deadlock; @fasterthanlime_deadlock].

### Behaviour-Oriented Concurrency

Behaviour-Oriented Concurrency (BOC) is a novel concurrency paradime [@when_concurrency_matters]

### Other concurrency paradimes

#### Shared Memory

#### Message Passing

#### Fork/Join

#### Actors

#### Structured Concurrency

### Verona Runtime

## The work done so far

So far, I've completed a basic rust bindings to the verona-rt library. Let's start with an example.

```rust
let string = CownPtr::new(String::new());
let vec = CownPtr::new(Vec::new());

when(&string, |mut s| {
    assert_eq!(&*s, "");
    s.push_str("foo");
});
when(&vec, |mut v| {
    assert_eq!(&*v, &[]);
    v.push(101);
});
when2(&string, &vec, |mut s, mut v| {
    assert_eq!(&*s, "foo");
    assert_eq!(&*v, &[101]);
    s.push_str("bar");
    v.push(666);
});
when(&string, |s| assert_eq!(&*s, "foobar"));
when(&vec, |v| assert_eq!(&*v, &[101, 666]));
```

This is very similar to the equivalent C++ code.

```cpp
auto string = make_cown<std::string>();
auto vec = make_cown<std::vector<int>>();

when(string) << [](auto s) {
  assert(*s == "");
  s->append("foo");
};
when(vec) << [](auto v) {
  assert(v->size() == 0);
  v->push_back(101);
};
when(string, vec) << [](auto s, auto v) {
  assert(*s == "foo");
  assert(*v == std::vector{101});
  s->append("bar");
  v->push_back(666);
};
when(string) << [](auto s) { assert(*s == "foobar"); };
when(vec) << [](auto v) { assert((*v == std::vector{101, 666})); };
```

## Design

- Resuse C++ Runtime
- Thin Rust Layer on top
- Chalenges
    - Generics across an FFI boundary
    - Aliasing when aquiring twice
    - Destructors
    - Move Ctors
- Who Owns RefCounting
    - Cost: Hard to inline refcounting
        - Potential fix: X-lang LTO
            - This complicates the build
            - Benchmark this
            - Benchmark rust vs C++
            - Savina benchmarks.
    - Cost: Store a dtor pointer in every object.
        - Potenial Fix: Pass dtor pointer on Drop call
        - Still has Nop call for Cowns with nop dtors
        - Althoug Xlang LTO could fix this (needs benchmarking).
    - Design Cost: We assume a load about the internals.
        - These arn't garenteed by the API.
        - Sometimes need a +9 object size.
        - Maintences concerns after I stop.
- Be no_std / have no_allocs
    - Lets you use their embeded runtime.
    - https://github.com/microsoft/monza

## Evaluation Plan

- Reuse benchmarks from BoC paper, to show low overhead.
- Subjectively, they also show something about expressiveness





## Ethical Issues

This project presents very few ethical issues. The only thing to be concerted
about is clear attribution of work, as it builds extensively on pre-existing
research into Behavior Oriented Concurrency, and it's implementation in the
Verona Runtime. It must be clear what of the work was my own, as what wasn't.

This is made more complex by the fact that I have a direct dependency on the
verona-rt library, and have made modification to it to support my specific use
case. I'll also (hopefully) end up conversing with MSR people about how to
integrate these changes upstream, and other possible directions. I need to be
careful to properly acknowledge this potential collaboration (should it occur).

## Bibliography

<!-- pandoc magically fills this in -->