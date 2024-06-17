

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

**Figure 4: Time to busy loop for $n$ µsecs**


To do this, I wrote a benchmark that would busy loop for a given length of time.
This would allow me to know how long a given benchmark "should" take, and
therefore see if this was replicated in the data.

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