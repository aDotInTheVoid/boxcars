---
title: "Boxcars: Behaviour Oriented Concurrency in Rust"
author: Alona Enraght-Moony
date: 2024-06-17
bibliography: ../cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
papersize: a4
geometry: margin=2cm
mainfont: CMU Serif
monofont: inconsolata
toc: true # TODO: Turn this false and place yourself.
toc-depth: 2
colorlinks: true
numbersections: true
---

# Abstract

Behaviour-Oriented Concurrency (BoC) is a novel concurrency paradime [@when_concurrency_matters]. I introduce Rust bindings to the verona-runtime.

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

## Schedular

# Implementation Chalenges

## Allocation Size Couroption

## TLS Destructors

# Evaluation

## Benchmarks

One important metric to evaluate the project on is performance. As discussed previously (§\ref{design-cowns}), it's not simple to call into the C++ runtime from rust, and I had to be somewhat indirect due to the FFI boundry. I wanted to measure the performance overhead of this, versus C++ code that can call it directly. 

Note: In all benchmarks below, `Rust` indicates using my `boxcars` library, whereas `C++` indicates using the `verona-rt` library directly.

All graphs were created using the excelent [`criterion`](https://github.com/bheisler/criterion.rs) (TODO: Cite?) library.

## Microbenchmarks

The first way to try to understand this would be with small microbenchmarks that
do just one thing. This would let us get a direct comparison for exactly
equivalent actions.

### Creating Cowns

![](./img/Create_Cowns.svg)
**Figure 1: Time to create $n$ cowns**

The first benchmark I wrote was to create a vector of $n$ `Cown`s, and then free them. This was to measure the overhead of Rust not being able to interact directly with the `Cown` constructors, but having to do it via FFI, as discussed previously (§\ref{design-cowns}).


```rust
let mut v = Vec::with_capacity(n);
for i in 0..n {
    v.push(Cown::<usize>::new(i));
}
```

```c++
std::vector<cown_ptr<size_t>> v;
v.reserve(n);

for (size_t i = 0; i < n; i++)
{
    v.push_back(make_cown<size_t>(i));
}
```

This overhead is measurable, but relativly small, with it only being 0.03ms when creating $2^{15}$ `Cown`s.

### Scheduling behaviours

![](./img/Schedule_Behaviours.svg)
**Figure 2: Time to schedule and run $n$ behaviours** 

The other major implementation difference is how behaviours are scheduled onto
the `Cown`s, with Rust also needing indirection over FFI (§\ref{design-behaviours}). Therefor
I benchmarked scheduling a large number of behaviours that do minimal work, to
measure the cost of the scheduling and exectution itself.

As shown, the FFI indirection imposes very little additional overhead over the C++ code which can interact directly with the runtime.

```rust
let c = Cown::new(0);
for _ in 0..n {
    when(&c, |mut c| {
        *c += 1;
    })
}
```

```c++
auto c = make_cown<int>(0);
for (int j = 0; j < n; j++)
{
    when(c) << [](auto c) { c++; };
}
```


### Setting up scheduler

![](./img/Scheduler.svg)

**Figure 3: Time to create and run a scheduler doing nothing**

The final thing to consider is how long it takes to set up the global scheduling
state. Both libraries consistantly take around 300μs to do so. In other
microbenchmarks, it was important to do this outside the loop being benchmarked,
to ensure that I was timing the code being benchmarked, and not having the
overhead of setting up and then tearing down the executors thread and memory
pool.

```rust
with_n_threads(1, || { /* no-op */ })
```

```cpp
Scheduler::get().init(1);
Scheduler::get().run();
```


### Busy looping

![](./img/Busy_Loop.svg)

**Figure 4: Time to busy loop for $n$ µsecs**

To do this, I wrote a benchmark that would busy loop for a given lenght of time.
This would allow me to know how long a given benchmark "should" take, and
therefor see if this was replicated in the data.

As the graph shows, it takes almost exactly $n$μs to run a behaviour that busy
loops for $n$ μs (as expected!). As all these times are much shorter than the
~300μs time to create the scheduler, this let me verify that my benchmarking was
only measuring the "work" getting done, and not anything else. 

```cpp
auto c = make_cown<size_t>(nsecs);    
when(c) << [](auto c) { busy_loop(*c); };    
```

```rust
let c = Cown::new(usecs);
when(&c, |c| unsafe {
    boxcars_busy_loop(*c);
})
```

## Subjective Things

- Manually Clone.
- Parial Borrors don't work
- C++ allows more expressive ctors `auto foo = make_cown<Foo>(a, b, c)`

![](./img/Busy_Loop.svg)

![](./img/Create_Cowns.svg)

![](./img/Fibonacci.svg)

![](./img/Schedule_Behaviours.svg)

It's fast.

# References