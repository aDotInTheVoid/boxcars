# Interim Report Planing

Submit in January

It's the first part of the final report.

- Related Work
- High Level Design
- Key Points of Contribution
- Sketch for the Evaluation


- Background:
    - Start with the BoC paper
    - Review of different sync primitives & concurrency paradimes
    - Background on the Rust language

- Design:
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


Background needs to be more detailed than design.
Design needs to include chalenges

## Random Technical Stuff

- Variadic when
    - Macros: Luke cheeseman
        - when1, when2, when3
    - Or use a trait:
        - when((foo, bar), |foo, bar| { ... })

        ```rust
        impl<T1, T1> WhenArgs for (CownPtr<T1>, CownPtr<T2>) {
            type Func<R> = fn(Aquired<T1>, Aquired<T2>) -> R;
        }
        ```