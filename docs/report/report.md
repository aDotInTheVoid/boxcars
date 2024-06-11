---
title: "Boxcars: Behaviour Oriented Concurrency in Rust"
author: Alona Enraght-Moony
date: 2024-06-17
bibliography: ../cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
papersize: a4
geometry: margin=3cm
mainfont: CMU Serif
monofont: inconsolata
toc: true # TODO: Turn this false and place yourself.
toc-depth: 2
colorlinks: true
numbersections: true
---

# Abstract

Behaviour-Oriented Concurrency (BoC) is a novel concurrency paradigm [@when_concurrency_matters]. I introduce Rust bindings to the verona-runtime.

# Acknowledgements

Marios Kogias, Mathiew Parkinson, David Chisnall, Sylvan Clebsch, Mara Bos, Nora

<!-- ```{=latex}
% TODO: Use this to place TOC after abstract.
{
\hypersetup{linkcolor=}
\setcounter{tocdepth}{3}
\tableofcontents
}
``` -->

# Introduction

# Background

# Design

## Cowns {#design-cowns}

## Behaviours {#design-behaviours}

## Behaviour API

## Scheduler

# Implementation Chalenges

## Allocation Size Couroption

## TLS Destructors

# Evaluation

## Benchmarks

One important metric to evaluate the project on is performance. As discussed previously (§\ref{design-cowns}), it's not simple to call into the C++ runtime from rust, and I had to be somewhat indirect due to the FFI boundary. I wanted to measure the performance overhead of this, versus C++ code that can call it directly. 

Note: In all benchmarks below, `Rust` indicates using my `boxcars` library, whereas `C++` indicates using the `verona-rt` library directly.

All graphs were created using the excellent [`criterion`](https://github.com/bheisler/criterion.rs) (TODO: Cite?) library.

### Microbenchmarks

The first way to try to understand this would be with small microbenchmarks that
do just one thing. This would let us get a direct comparison for exactly
equivalent actions.

#### Creating Cowns


```{=latex}
\mbox{}\\
```

![](./img/Create_Cowns.svg)

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

#### Scheduling behaviours


```{=latex}
\mbox{}\\
```

![](./img/Schedule_Behaviours.svg)

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


#### Setting up scheduler

```{=latex}
\mbox{}\\
```

![](./img/Scheduler.svg)

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


#### Busy looping


![](./img/Busy_Loop.svg)


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

### Larger Benchmarks

Performance is kind of mixed.

![](./img/savina_Banking.svg)
![](./img/savina_Barber.svg)
![](./img/Fibonacci.svg)

## User Experience

Of course, performance isn't the only thing worth measuring. It's also worth
considering what it's like to *use* these libraries, and how the language and
API differences impact what it's like to write BoC code in them.

### Parial Borrows

Most of the type `AcquiredCown<'a, T>` acts like `&'a mut T`, and users don't have to worry about the fact that it's actually a smart pointer wrapper.

```rust
fn print_str(s: &str) {
    println!("{s}");
}

let cown = Cown::<&str>::new("hello world");
when(&cown, |acq_cown: AcquiredCown<&str>| {
    print_str(&acq_cown);
});
```

This is done by implementing the
[`std::operator::Deref`](https://doc.rust-lang.org/stable/std/ops/trait.Deref.html)
and [`DerefMut`](https://doc.rust-lang.org/stable/std/ops/trait.DerefMut.html)
trait. This is broadly equivalent to `verona::cpp::acquired_cown` overloading
`operator->`, `operator*` and `operator T&`.

However, because this means that the compiller will implicity insert calls to our `Deref` implemnations ("deref coercion" [@rust_book]), this makes things much harder for the borrow checker [^borrow_cant_see_into].


[^borrow_cant_see_into]: Remember, when checking a function, the borrow-checker
    can't see the bodies of other functions, only their signatures.

Consider the following code:

```rust
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

On the last line, `foo` is borrowed mutably twice. However, because these are
both borrows of different fields of `foo`, neither one aliases with each other,
so we haven't violated the core principle of Aliasing XOR Mutation. This feature, where you can borrow single fields of a struct without borrowing the whole struct, is called partial borrows [@nomicon].

However, when we attempt this same thing on a `AcquiredCown<Foo>`, it doesn't work:

```rust
let cown = Cown::new(Foo { a: 1, b: 1 });
when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
    use_ints(&mut acq_cown.a, &mut acq_cown.b)
});
```

```
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

### Compiler Errors

Because Rust checks generics at the call site, rather than using template
expansion, is can often produce much nicer error messages than the equivalent
from C++.

(Note: comparing `g++ 13.2.0` to `rustc 1.78.0`)

#### Wrong Cown constructor.

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

#### Wrong Args

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

#### Rustc being helpfull and unhelpfull

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

### auto-copy vs explicit clone.

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

# References