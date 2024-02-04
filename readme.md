# Behavior Oriented Concurrency in Rust.

```rust
use verona_rt::{
    cown::CownPtr,
    scheduler::with,
    when::{when, when2},
};

with(|| {
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
});
```

For an introduction to the Behavior Oriented Concurrency model, see the
[OOPSLA 2023 Paper](https://dl.acm.org/doi/10.1145/3622852). This library
aims to provide idiomatic Rust bindings to the [Verona runtime](https://github.com/microsoft/verona-rt),
as introduced in that paper.

## Current Status

This is a research project, and is at an early stage of development. It is not
ready for use outside of research.
