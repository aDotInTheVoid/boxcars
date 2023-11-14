# Systematic Testing

The `systematic_testing` cargo features will build `verona-rt` with `USE_SYSTEMATIC_TESTING` defined.

```
cargo -v t -p verona-rt --features systematic_testing -- leak_detector_new
```

## Using the logger.

First enable the logger.

```rust
unsafe {
    verona_rt_sys::enable_logging();
}
```

This is global racy state, so should probably only be used temporarily.

```rust
verona_rt::log::log("Yoohoo, we're here");
```

This may also be racy, so be careful.