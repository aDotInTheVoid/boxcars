# Behavior Oriented Concurrency in Rust.

For an introduction to the Behavior Oriented Concurrency model, see the
[OOPSLA 2023 Paper](https://dl.acm.org/doi/10.1145/3622852). This library
aims to provide idiomatic Rust bindings to the [Verona runtime](https://github.com/microsoft/verona-rt),
as introduced in that paper.

## Current Status

This is a research project, and is at an early stage of development. It is not
ready for use outside of research.

## Developing

Cargo will automatically invoke cmake. However, when working on the bindings,
it's nicer to use it manually.

```
cd crates/verona-rt-sys/cpp
cmake -B build -GNinja -D VERONA_RT_ONLY_HEADER_LIBRARY=ON
ninja -C build
```