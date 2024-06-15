---
title: "Boxcars: Behaviour Oriented Concurrency in Rust"
author: Alona Enraght-Moony
date: 2024-06-17
bibliography: ../cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
papersize: a4
geometry: a4paper,hmargin=2.8cm,vmargin=2.0cm,includeheadfoot
toc: false # We insert the TOC ourselves, after the abstact and acknoledgements.
toc-depth: 2
colorlinks: true
numbersections: true
documentclass: report
header-includes: |
  ```{=latex}
  \input{headers.tex}
  ```
---


```{=latex}
\input{title.tex}
\begin{abstract}
```

The Rust programming language is pretty cool. However, it could be better. What if it was good?

```{=latex}
\end{abstract}
\renewcommand{\abstractname}{Acknowledgements}
\begin{abstract}
```

First and foremost, I must thanks Marios Kogias for his excelent job as project
supervisor.

I'd also like to thank Mathiew Parkinson, David Chisnall, Sylvan Clebsch for
providing critical feedback when presented with a much earlier version of the
design, and pointing out a different direction to explore that ended up being
much more fruitful than my initial attempts.

Finally I'd like to thank Nora for pointing out to me how to make
ThreadSanitizer work with Rust's standard library, and Mara Bos for chiming in
at the ideal time to point out a subtle interaction with thead-local
destructors.

```{=latex}
\end{abstract}
{
\hypersetup{linkcolor=}
\setcounter{tocdepth}{3}
\tableofcontents
}
```

# Introduction

Help!!!

```cpp {caption="Foo Code" label="code:foo"}
int main() {
    std::cout << "Lol";
}
```

```java {caption="Bar Code" label="code:bar"}
class Bar {}
```

Foo = \ref{code:foo}. Bar = \ref{code:bar}.


## Why



## How?

# Background

## Rust

Rust [@rust_book] is a programming language originally developed by Mozilla Research,
and currently maintained by a large cross-org team.

Rust's most important feature (for our purposes) is its system of **Ownership & Borrowing**.

### Ownership & Borrowing: A very fast introduction.

(A full tutorial on ownership and borrowing is beyond the scope of this report. See [@rust_book] for details.)
The core idea of **ownership** is that each value has a unique owner, and that value is dropped when the
owner goes out of scope. To quote The Book [@rust_book]:

> - Each value in Rust has an owner.
> - There can only be one owner at a time.
> - When the owner goes out of scope, the value will be dropped.

However with only these rules, programming in Rust would be extremely
uneconomic. If a function took a string as a parameter, it would have "take
ownership" of that value, so the calling function could no longer use it.
[^return_values]

[^return_values]: This could be circumvented by having a function return back it's args to the caller, but this would
be extremely cumbersome.

Therefore, Rust introduces the additional notion of **borrowing**. A value can be borrowed in
one of two ways: by a shared reference (spelt `&T`), or an exclusive reference (`&mut T`). These are
also sometimes referred to as a immutable or mutable reference respectively [@dtolnay_ref].

A value may have many shared references to it at a given time, but if it has any
exclusive reference to it that reference must be the only one. With a shared
reference, you can only read from the value. An exclusive reference is required
to mutate it. More succinctly, Rust references are "Aliasable XOR mutable" [@boats_smaller].

Borrowed values have a **lifetime** for which they are borrowed. This is needed to ensure
that all exclusive references don't overlap with shared ones.

```rust
let x: i32 = 0;

let shared_1: &i32 = &x; // Lifetime 1 starts
dbg!(shared_1);
// Lifetime 1 stops.

let exclusive_2: &mut i32 = &mut x; // Lifetime 2 starts
// Would be compiler error to borrow x here.
*exclusive_2 += 10;
// Lifetime 2 ends

// But now that lifetime 2 is over, we can borrow again

let shared_3: &i32 = &x; // Lifetime 3 starts
let shared_4: &i32 = &x; // Lifetime 4 starts

dbg!(shared_4);
// Lifetime 4 ends
dbg!(shared_3);
// Lifetime 3 ends
```

This is also used to ensure that references don't outlive the objects they borrow.

```rust
let mut x: &i32 = 0;
{
    let short_lived: i32 = 0;
    x = &short_lived;
} // short_lived goes out of scope here.
dbg!(x);
```

will error with

```
error[E0597]: `short_lived` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let short_lived: i32 = 0;
  |             ----------- binding `short_lived` declared here
5 |         x = &short_lived;
  |             ^^^^^^^^^^^^ borrowed value does not live long enough
6 |     }
  |     - `short_lived` dropped here while still borrowed
7 |     dbg!(x);
  |          - borrow later used here
```

instead of doing UB like it would in C++.

### Consequence of this: No iterator invalidation

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

does undefined behavior.

This is because when we call `.push_back()`, the vector needs to reallocate, as it initially
only had enough capacity for 3 elements. After moving the values to the new allocation, it frees
the old one. This means `x` is a dangling pointer, and dereferencing it is a heap use-after-free.

Indeeed, we can see this by using Asan, which can show up a stack trace of when this happend.

```
=================================================================
==26780==ERROR: AddressSanitizer: heap-use-after-free on address 0x602000000010 at pc 0x55dc3792032f bp 0x7ffcdf94ec50 sp 0x7ffcdf94ec48
READ of size 4 at 0x602000000010 thread T0
    #0 0x55dc3792032e in main /home/alona/tmp/./bad.cpp:8:18
   
0x602000000010 is located 0 bytes inside of 12-byte region [0x602000000010,0x60200000001c)
freed by thread T0 here:
    #0 0x55dc3791e131 in operator delete(void*) (/home/alona/tmp/a.out+0xf6131) (BuildId: ba94dc30bfb525b0467634abdb5f45d077025f55)
    #6 0x55dc379207fc in std::vector<int, std::allocator<int>>::push_back(int&&) /usr/bin/../lib/gcc/x86_64-linux-gnu/13/../../../../include/c++/13/bits/stl_vector.h:1296:9
    #7 0x55dc379202de in main /home/alona/tmp/./bad.cpp:7:8
  
previously allocated by thread T0 here:
    #0 0x55dc3791d8b1 in operator new(unsigned long) (/home/alona/tmp/a.out+0xf58b1) (BuildId: ba94dc30bfb525b0467634abdb5f45d077025f55)
    #5 0x55dc379206bd in std::vector<int, std::allocator<int>>::vector(std::initializer_list<int>, std::allocator<int> const&) /usr/bin/../lib/gcc/x86_64-linux-gnu/13/../../../../include/c++/13/bits/stl_vector.h:679:2
    #6 0x55dc37920234 in main /home/alona/tmp/./bad.cpp:5:22
```

We allocated some memory to create the vector, took a reference to it,
 but then freed it when we did the `push_back()`, and
then used the it afterwards.



Currently on my machine it prints a seemingly random value.
However modifying to so that the vector is larger (`std::vector<int> xs (1000000, -1);`),
makes the program segfault instead. The exact details of what happens here are at the whims
of the compiler and the standard library

### Consequence of this: Fearless concurrency

```Rust
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
One can trivially perform one by acquiring two mutex's in different orders:

```Rust
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

## Behaviour-Oriented Concurrency

Behaviour-Oriented Concurrency (BoC) is a novel concurrency paradigm
[@when_concurrency_matters]. It has 2 key components:

- **Cowns**: A cown (short for concurrent owner) is a piece of data.

    A cown can be in one of two states: available or acquired. An available
    cown is eligible to be acquired by a behaviour, but it's associated data
    cannot be accessed. An acquired cown can have it's data accessed, but only
    while it's aquired.

    The only way to acquire a cown (and thus access it's) is to run a behaviour on it.

- **Behaviours**: A behaviour is a unit of execution that acts upon a set of cowns.

    When you create a behaviour, you give the set of cowns it acts on, as well as
    the code to run. When all the required cowns can be acquired, the code is executed, and then
    the cowns are made available again.

    Cowns can only be acquired by one behaviour at once. This can be thaugh of as being a bit
    like each Cown having a mutex, which is locked before the behaviour starts and unlocked after
    it ends. However, every cown in a behaviour is acquired "at once"

    Note that this means that creating a behaviour returns immediately, and the code inside
    will be executed at some indetermined future point, when unique access to all cowns can
    be guaranteed.


```scala
var myCown: Cown[int] = cown.create(10);

when(myCown) {
    myCown += 10;    
}

myCown += 10; // invalid.
```

BoC is both *data-race free* and *deadlock free*. No data races can occur, as
cowns can only be modified when acquired by behaviours,

## Verona Runtime

The verona runtime (sometimes also known as `verona-rt`) is a C++ library that
implements behaviour oriented concurrency. It is intended to be a part of the
currently in development Verona language, but it can also be used as a
freestanding C++ library today.

It uses the C++ type system to enforse the destinction between availible and 
aquired cowns. The `cown_ptr<T>` type references a cown containing `T`, but doens't
allow accessing the data. Instead, when a behaviour executed, it's given an `acquired_cown<T>`, 
which can access the underlying data.

Internally, these have the same representation, but. An `acquired_cown<T>` is
only handed out when the behavior runs. In this way, the compiler can enforce that
cowns can only have their data accessed when acquired. Eg:

```cpp
cown_ptr<int> my_cown = make_cown<int>(10);
when(my_cown) << [](acquired_cown<int> my_cown) {
    my_cown += 10; // this works fine, via operator overloading.
};
my_cown += 10; // compiler error!
```

The `when` function takes a `cown_ptr`, and a lambda that takes an `acquired_cown`.
When the cown can be acquired, it creates an `acquired_cown` (which cannot be done outside the verona-rt library),
and passes that to the lambda. 

However, a determined user can circumvent this type safety. As one example:

```cpp
cown_ptr<int> my_cown = make_cown<int>(10);
int* escape_data;
when(my_cown) << [&escape_data](acquired_cown<int> my_cown) {
    escape_data = &*my_cown;
};
sleep(3); // Wait for behaviour to run
*escape_data += 10; // Modifies cown without acquiring!!!
```

This exposes the underlying data via `escape_data`. A user could then
modify the cowns data without acquiring it, undermining BoC's goal
of no-data-races. This is because C++ has no mechanism to limit
how long a pointer can be used, and the `acquired_cown` class
must give out pointers to allow access to the underlying data.

Rust can solve this with the power of lifetimes.

<!--
### Other concurrency paradimes

- Shared Memory
- Message Passing
- Fork/Join
- Actors
- Structured Concurrency
-->

# Design

## Cowns {#design-cowns}

The design of the Rust `Cown` API is as follows [^phantom]: 

[^phantom]: `Cown` also has a `_marker: PhantomData<Mutex<T>>` field, but that's
    not relavent here.

```rust
pub struct Cown<T> {
   cown_ptr: CownPtr,
}

#[repr(transparent)]
pub struct CownPtr {
   addr: *mut (),
}
```

![](./img/cown-layout.png)

## Behaviours {#design-behaviours}

## Behaviour API

## Scheduler


# Performance Evaluation.

One important metric to evaluate the project on is performance. As discussed previously (§\ref{design-cowns}), it's not simple to call into the C++ runtime from rust, and I had to be somewhat indirect due to the FFI boundary. I wanted to measure the performance overhead of this, versus C++ code that can call it directly. 

Note: In all benchmarks below, `Rust` indicates using my `boxcars` library, whereas `C++` indicates using the `verona-rt` library directly.

All graphs were created using the excellent [`criterion`](https://github.com/bheisler/criterion.rs) (TODO: Cite?) library.

## Microbenchmarks

The first way to try to understand this would be with small microbenchmarks that
do just one thing. This would let us get a direct comparison for exactly
equivalent actions.

### Creating Cowns


![](./bench_graphs/Create_Cowns.svg)

**Figure 1: Time to create $n$ cowns**

The first benchmark I wrote was to create a vector of $n$ `Cown`s, and then free them. This was to measure the overhead of Rust not being able to interact directly with the `Cown` constructors, but having to do it via FFI, as discussed previously (§\ref{design-cowns}).


```rust
// Rust
let mut v = Vec::with_capacity(n);
for i in 0..n {
    v.push(Cown::<usize>::new(i));
}
```

```c++
// C++
std::vector<cown_ptr<size_t>> v;
v.reserve(n);

for (size_t i = 0; i < n; i++)
{
    v.push_back(make_cown<size_t>(i));
}
```

This overhead is measurable, but relatively small, with it only being 0.03ms when creating $2^{15}$ `Cown`s.

### Scheduling behaviours

![](./bench_graphs/Schedule_Behaviours.svg)

**Figure 2: Time to schedule and run $n$ behaviours** 

The other major implementation difference is how behaviours are scheduled onto
the `Cown`s, with Rust also needing indirection over FFI (§\ref{design-behaviours}). Therefor
I benchmarked scheduling a large number of behaviours that do minimal work, to
measure the cost of the scheduling and execution itself.

As shown, the FFI indirection imposes very little additional overhead over the C++ code which can interact directly with the runtime.

```rust
// Rust
let c = Cown::new(0);
for _ in 0..n {
    when(&c, |mut c| {
        *c += 1;
    })
}
```

```c++
// C++
auto c = make_cown<int>(0);
for (int j = 0; j < n; j++)
{
    when(c) << [](auto c) { c++; };
}
```


### Setting up scheduler

```{=latex}
\mbox{}\\
```

![](./bench_graphs/Scheduler.svg)

**Figure 3: Time to create and run a scheduler doing nothing**

The final thing to consider is how long it takes to set up the global scheduling
state. Both libraries consistently take around 300μs to do so. In other
microbenchmarks, it was important to do this outside the loop being benchmarked,
to ensure that I was timing the code being benchmarked, and not having the
overhead of setting up and then tearing down the executors thread and memory
pool.

```rust
// Rust
with_n_threads(1, || { /* no-op */ })
```

```cpp
// C++
Scheduler::get().init(1);
Scheduler::get().run();
```


### Busy looping


![](./bench_graphs/Busy_Loop.svg)


```{=latex}
\mbox{}\\
```


**Figure 4: Time to busy loop for $n$ µsecs**


To do this, I wrote a benchmark that would busy loop for a given length of time.
This would allow me to know how long a given benchmark "should" take, and
therefor see if this was replicated in the data.

As the graph shows, it takes almost exactly $n$μs to run a behaviour that busy
loops for $n$ μs (as expected!). As all these times are much shorter than the
~300μs time to create the scheduler, this let me verify that my benchmarking was
only measuring the "work" getting done, and not anything else. 

```rust
// Rust
let c = Cown::new(usecs);
when(&c, |c| unsafe {
    boxcars_busy_loop(*c);
})
```

```cpp
// C++
auto c = make_cown<size_t>(nsecs);    
when(c) << [](auto c) { busy_loop(*c); };    
```

## Larger Benchmarks

Performance is kind of mixed.

![](./bench_graphs/savina_Banking.svg)
![](./bench_graphs/savina_Barber.svg)
![](./bench_graphs/Fibonacci.svg)

# Evaluation of User Experience

Of course, performance isn't the only thing worth measuring. It's also worth
considering what it's like to *use* these libraries, and how the language and
API differences impact what it's like to write BoC code in them.

## Parial Borrows

While most of the time `AcquiredCown<'a, T>` acts like a transparent wrapper
over `&'a mut T`, there are some cases where this abstraction becomes leaky, and
users need to be aware that they're not dealing with a normal reference.  


```rust {label="autoderef-example" caption="Demonstration of auto-deref"}
    let cown = Cown::<i32>::new(10);

    // This type anotation isn't needed, but is here to make the coercion clearer.
    //                     vv
    when(&cown, |acq_cown: AcquiredCown<i32>| {
        let n: i32 = *acq_cown; 
    });
```

In code like that in listing \ref{autoderef-example}, `acq_cown` acts like an `&mut i32`, and is able to be dereferenced into `i32`. This even extends to method calls, as shown in listing \ref{autoderef-methods}, where we can call `&str`s `to_uppercase` method on an `AcquiredCown<&str>`.

```rust {label="autoderef-methods" caption="Calling methods on acquired cowns via auto-deref"}
    let cown = Cown::<&str>::new("hello");

    when(&cown, |acq_cown: AcquiredCown<&str>| {
        let uppercase_str = acq_cown.to_uppercase();
    });
```

This is possible because `AcquiredCown` implements the
[`Deref`](https://doc.rust-lang.org/stable/std/ops/trait.Deref.html) and
[`DerefMut`](https://doc.rust-lang.org/stable/std/ops/trait.DerefMut.html)
traits. The rust compiller will implicitly insert calls to these methods, to
allow a `AcquiredCown<T>` to be treated like an `&mut T`. This normally works
seamlessly, as shown in listings \ref{autoderef-example} and
\ref{autoderef-methods}.


However, because this means that the compiller will implicity insert calls to our `Deref` implemnations ("deref coercion" [@rust_book]), this makes things much harder for the borrow checker [^borrow_cant_see_into].


[^borrow_cant_see_into]: Remember, when checking a function, the borrow-checker
    can't see the bodies of other functions, only their signatures.


```rust {label="code:partial-works" caption="Demonstration of partial borrowing"}
struct Foo {
    a: i32,
    b: i32,
}

fn use_ints(x: &mut i32, y: &mut i32) {
    *x += 1;
    *y += 1;
}

let mut foo = Foo { a: 1, b: 1 };
use_ints(&mut foo.a, &mut foo.b);
```

On the last line of \ref{code:partial-works}, `foo` is borrowed mutably twice.
However, because these are both borrows of different fields of `foo`, neither
one aliases with each other, so we haven't violated the core principle of
Aliasing XOR Mutation. This feature, where you can borrow single fields of a
struct without borrowing the whole struct, is called partial borrows [@nomicon].

However, when we attempt this same thing on a `AcquiredCown<Foo>`, it doesn't work:

```rust {label="needs-partial" caption="Attempting to borrow two fields of a struct in a cown."}
let cown = Cown::new(Foo { a: 1, b: 1 });
when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
    use_ints(&mut acq_cown.a, &mut acq_cown.b)
});
```

```{caption="Compiller error of listing \ref{needs-partial}"}
error[E0499]: cannot borrow `acq_cown` as mutable more than once at a time
  --> tests/ui/partial-borrow.rs:17:44
   |
17 |     use_ints(&mut acq_cown.a, &mut acq_cown.b)
   |     --------      --------         ^^^^^^^^ second mutable borrow occurs here
   |     |             |
   |     |             first mutable borrow occurs here
   |     first borrow later used by call
```

Why is this? The compiller has implicitly inserted call to the `deref_mut` function to convert from `AcquiredCown<Foo>` to `&mut Foo`, so what the borrow checker runs on looks like:

```rust
use_ints(
    &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).a,
    &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).b,
)
```

Here it's clear that the problem is that both `deref_mut` calls *must* be able
to borrow acq_cown, and indeed, that the error we get:

```
error[E0499]: cannot borrow `acq_cown` as mutable more than once at a time
  --> tests/ui/partial-borrow.rs:30:65
   |
28 |  use_ints(
   |  -------- first borrow later used by call
29 |      &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).a,
   |                                                      ------------- first mutable borrow occurs here
30 |      &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).b,
   |                                                      ^^^^^^^^^^^^^ second mutable borrow occurs here
```

The borrow checker has no way of knowing (or isn't _allowed_) to know, that our
`deref_mut` is just a pointer dereference, and that because we then access
disjoint struct field, this should be allowed as a partial borrow.

However, we can work around this:

```rust
when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
    let mut_ref: &mut Foo = &mut *acq_cown;
    use_ints(&mut mut_ref.a, &mut mut_ref.b)
});
```

Here all the deref coersion happens on the second line, and it only happens once
there to make `mut_ref`. Then on the third line, we partial borrow `mut_ref` to
get the two fields we want. This is allowed, because there's no hidden function
calls, so the borrow checker can locally check that we're obeying Aliasing XOR
Mutation.

A potential workaround here would be to have the closure in `when` take `&mut T`
instead of `AcquiredCown<T>`. This would make parial borrows work *by default*.
However, this would mean that we'd loose the information that this was a
reference to a cown (and not just any data). This is unfortunate, as that means
we would no longer be able to schedule a new behaviour onto an acquired cown.
Therefor I decided not to make this change, as while it does make partial
borrows nicer, they can already be done with clunkier syntax, and it would mean
giving up on important functionality.

## Compiler Errors

Because Rust checks generics at the call site, rather than using template
expansion, is can often produce much nicer error messages than the equivalent
from C++.

(Note: comparing `g++ 13.2.0` to `rustc 1.78.0`)

### Wrong Cown constructor.

The most frequently encountered one of these when writing the benchmarks was getting 

```c++
class Foo
{
  int number_;
  const char* str_;

public:
  Foo(int number, const char* str) : number_(number), str_(str) {}
};

// These are the wrong way round
auto c_foo = make_cown<Foo>("hello", 101);
```


```
In file included from verona-rt/src/rt/./cpp/when.h:6,
                 from cpp/playground.cc:1:
verona-rt/src/rt/./cpp/cown.h: In instantiation of ‘verona::cpp::ActualCown<T>::ActualCown(Args&& ...) [with Args = {const char (&)[6], int}; T = Foo]’:
verona-rt/src/rt/./cpp/cown.h:374:24:   required from ‘verona::cpp::cown_ptr<T> verona::cpp::make_cown(Args&& ...) [with T = Foo; Args = {const char (&)[6], int}]’
cpp/playground.cc:27:30:   required from here
verona-rt/src/rt/./cpp/cown.h:50:32: error: invalid conversion from ‘const char*’ to ‘int’ [-fpermissive]
   50 |     ActualCown(Args&&... ts) : value(std::forward<Args>(ts)...)
      |                                ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
      |                                |
      |                                const char*
cpp/playground.cc:21:11: note:   initializing argument 1 of ‘Foo::Foo(int, const char*)’
   21 |   Foo(int number, const char* str) : number_(number), str_(str) {}
      |       ~~~~^~~~~~
verona-rt/src/rt/./cpp/cown.h:50:32: error: invalid conversion from ‘int’ to ‘const char*’ [-fpermissive]
   50 |     ActualCown(Args&&... ts) : value(std::forward<Args>(ts)...)
      |                                ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
      |                                |
      |                                int
cpp/playground.cc:21:31: note:   initializing argument 2 of ‘Foo::Foo(int, const char*)’
   21 |   Foo(int number, const char* str) : number_(number), str_(str) {}
      | 
```

Whereas in Rust if you make the same mistake:

```rust
struct Foo {
    number: i32,
    string: &'static str,
}

impl Foo {
    fn new(number: i32, string: &'static str) -> Self {
        Self { number, string }
    }
}

let c_foo = Cown::new(Foo::new("hello", 101));
```

You instead get:

```
error[E0308]: arguments to this function are incorrect
  --> crates/verona-rt/examples/err.rs:15:27
   |
15 |     let c_foo = Cown::new(Foo::new("hello", 101));
   |                           ^^^^^^^^ -------  --- expected `&'static str`, found `{integer}`
   |                                    |
   |                                    expected `i32`, found `&'static str`
   |
note: associated function defined here
  --> crates/verona-rt/examples/err.rs:10:12
   |
10 |         fn new(number: i32, string: &'static str) -> Self {
   |            ^^^ -----------  --------------------
help: swap these arguments
   |
15 |     let c_foo = Cown::new(Foo::new(101, "hello"));
   |                                   ~~~~~~~~~~~~~~

For more information about this error, try `rustc --explain E0308`.
```

Which is much more understandable, as it doesn't need to take a detour through the templated libary code.
<!-- TODO: More here about not needing std::forward -->

### Wrong Args

Another case to consider is getting the arguments to the behaviour wrong.

```c++
auto a = make_cown<uint32_t>(101);
when(a) << [](acquired_cown<bool> a) {};
```

This causes C++ compillers to spew out the internals of the `when` implementation, because the type signature isn't part of then `when` function:

```
In file included from cpp/playground.cc:1:
verona-rt/src/rt/./cpp/when.h: In instantiation of ‘auto verona::cpp::When<F, Args>::to_tuple() [with F = real_main()::<lambda(verona::cpp::acquired_cown<bool>)>; Args = {verona::cpp::Access<unsigned int>}]’:
verona-rt/src/rt/./cpp/when.h:181:28:   required from ‘void verona::cpp::Batch<Args>::create_behaviour(verona::rt::BehaviourCore**) [with long unsigned int index = 0; Args = {verona::cpp::When<real_main()::<lambda(verona::cpp::acquired_cown<bool>)>, verona::cpp::Access<unsigned int> >}]’
verona-rt/src/rt/./cpp/when.h:204:25:   required from ‘verona::cpp::Batch<Args>::~Batch() [with Args = {verona::cpp::When<real_main()::<lambda(verona::cpp::acquired_cown<bool>)>, verona::cpp::Access<unsigned int> >}]’
verona-rt/src/rt/./cpp/when.h:486:16:   required from ‘auto verona::cpp::PreWhen<Args>::operator<<(F&&) [with F = real_main()::<lambda(verona::cpp::acquired_cown<bool>)>; Args = {verona::cpp::Access<unsigned int>}]’
cpp/playground.cc:19:41:   required from here
verona-rt/src/rt/./cpp/when.h:398:27: error: no match for call to ‘(std::remove_reference<real_main()::<lambda(verona::cpp::acquired_cown<bool>)>&>::type {aka real_main()::<lambda(verona::cpp::acquired_cown<bool>)>}) (verona::cpp::acquired_cown<unsigned int>)’
  398 |               std::move(f)(access_to_acquired<typename Args::Type>(args)...);
      |               ~~~~~~~~~~~~^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
verona-rt/src/rt/./cpp/when.h:398:27: note: candidate: ‘void (*)(verona::cpp::acquired_cown<bool>)’ (conversion)
verona-rt/src/rt/./cpp/when.h:398:27: note:   candidate expects 2 arguments, 2 provided
cpp/playground.cc:19:14: note: candidate: ‘real_main()::<lambda(verona::cpp::acquired_cown<bool>)>’
   19 |   when(a) << [](acquired_cown<bool> a) {};
      |              ^
cpp/playground.cc:19:14: note:   no known conversion for argument 1 from ‘acquired_cown<unsigned int>’ to ‘acquired_cown<bool>’
```

Rust

```rust
let a = Cown::<u32>::new(101);
when(&a, |a: AcquiredCown<bool>| {});
```

which gives:

```
error[E0631]: type mismatch in closure arguments
  --> crates/verona-rt/examples/err.rs:20:5
   |
20 |     when(&a, |a: AcquiredCown<bool>| {});
   |     ^^^^^^^^^-----------------------^^^^
   |     |        |
   |     |        found signature defined here
   |     expected due to this
   |
   = note: expected closure signature `for<'a> fn(AcquiredCown<'a, u32>) -> _`
              found closure signature `fn(AcquiredCown<'_, bool>) -> _`
note: required by a bound in `when`
  --> /home/alona/dev2/boxcars/crates/verona-rt/src/variadic_when.rs:14:8
   |
11 | pub fn when<C, F>(cowns: C, func: F)
   |        ---- required by a bound in this function
...
14 |     F: for<'a> FnOnce(C::Acquired<'a>) + Send + 'static,
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `when`

For more information about this error, try `rustc --explain E0631`.
```

### Rustc being helpfull and unhelpfull

```rust
let foo = Cown::new(101);
let bar = Cown::new(202);

when((&foo, &bar), |foo, bar| {});
```

gives:

```
error[E0593]: closure is expected to take a single 2-tuple as argument, but it takes 2 distinct arguments
  --> crates/verona-rt/examples/err.rs:22:5
   |
22 |     when((&foo, &bar), |foo, bar| {});
   |     ^^^^^^^^^^^^^^^^^^^----------^^^^
   |     |                  |
   |     |                  takes 2 distinct arguments
   |     expected closure that takes a single 2-tuple as argument
   |
help: change the closure to accept a tuple instead of individual arguments
   |
22 |     when((&foo, &bar), |(foo, bar)| {});
   |  
```

However, not all cases can be caught, and you definatly can still get unhelpfull error messages:

```rust
let foo = Cown::new(101);
let bar = Cown::new(202);

when((foo, bar), |(foo, bar)| {});
```

Which produces:

```
error[E0277]: the trait bound `(Cown<{integer}>, Cown<{integer}>): CownCollection` is not satisfied
  --> crates/verona-rt/examples/err.rs:22:10
   |
22 |     when((foo, bar), |(foo, bar)| {});
   |     ---- ^^^^^^^^^^ the trait `CownCollection` is not implemented for `(Cown<{integer}>, Cown<{integer}>)`
   |     |
   |     required by a bound introduced by this call
   |
   = help: the following other types implement trait `CownCollection`:
             (&Cown<A>, &Cown<B>)
             (&Cown<A>, &Cown<B>, &Cown<C>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>, &Cown<H>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>, &Cown<H>, &Cown<I>)
note: required by a bound in `when`
  --> /home/alona/dev2/boxcars/crates/verona-rt/src/variadic_when.rs:13:8
   |
11 | pub fn when<C, F>(cowns: C, func: F)
   |        ---- required by a bound in this function
12 | where
13 |     C: CownCollection,
   |        ^^^^^^^^^^^^^^ required by this bound in `when`

error[E0277]: expected a `FnOnce(<(Cown<{integer}>, Cown<{integer}>) as CownCollection>::Acquired<'a>)` closure, found `_`
  --> crates/verona-rt/examples/err.rs:22:22
   |
22 |     when((foo, bar), |(foo, bar)| {});
   |     ----             ^^^^^^^^^^^^^^^ expected an `FnOnce(<(Cown<{integer}>, Cown<{integer}>) as CownCollection>::Acquired<'a>)` closure, found `_`
   |     |
   |     required by a bound introduced by this call
   |
   = help: the trait `CownCollection` is not implemented for `_`
   = help: the following other types implement trait `CownCollection`:
             (&Cown<A>, &Cown<B>)
             (&Cown<A>, &Cown<B>, &Cown<C>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>, &Cown<H>)
             (&Cown<A>, &Cown<B>, &Cown<C>, &Cown<D>, &Cown<E>, &Cown<F>, &Cown<G>, &Cown<H>, &Cown<I>)
note: required by a bound in `when`
  --> /home/alona/dev2/boxcars/crates/verona-rt/src/variadic_when.rs:14:8
   |
11 | pub fn when<C, F>(cowns: C, func: F)
   |        ---- required by a bound in this function
...
14 |     F: for<'a> FnOnce(C::Acquired<'a>) + Send + 'static,
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `when`

For more information about this error, try `rustc --explain E0277`.
```

The root of the problem is that you need to borrow these `Cown`s to form a
`CownCollection`. However, becasue `when`s first argument is generic, `rustc`
can't say what type it needs to be, only that it must implement a certain trait.
This also causes a followup error, where it claims that the function has the
wrong signature, because it can't find the correct one.

## auto-copy vs explicit clone.

In C++, the `cown_ptr` class overloads it's copy and assignment constructor to automaticly update the reference count:

```c++
// class cown_ptr {

// Copy an existing cown ptr.  Shares the underlying cown.
cown_ptr(const cown_ptr& other)
{
    allocated_cown = other.allocated_cown;
    if (allocated_cown != nullptr)
        verona::rt::Cown::acquire(allocated_cown);
}

// Copy an existing cown ptr.  Shares the underlying cown.
cown_ptr& operator=(const cown_ptr& other)
{
    clear();
    allocated_cown = other.allocated_cown;
    if (allocated_cown != nullptr)
        verona::rt::Cown::acquire(allocated_cown);
    return *this;
}
```

This means that the following code Just Works:

```c++
auto c1 = make_cown<int>(10);
auto c2 = c1; // copy constuctor
c2 = c1; // copy-assignment operator
```

And all the `=` magically update the reference count to be correct.

Whereas in rust, because `=` moves (and there's no way to overload it), you need to instead write:

```rust
let c1 = Cown::new(10);
let mut c2 = c1.clone()
c2 = c1;
```

This adds visual clutter but also means that it's clearer when you're paying to
cost to do reference counting.

# Future Works

- Large scale software in BoC
- Passing datatypes between C++ and Rust.

# References