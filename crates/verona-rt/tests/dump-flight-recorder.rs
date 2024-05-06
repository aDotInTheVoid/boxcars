use std::ffi::CString;

use verona_rt::{log, with_scheduler};

// cargo test --test dump-flight-recorder --features flight_recorder

#[test]
#[cfg_attr(
    feature = "sanitizer_address",
    ignore = "we intentionally leak memory here"
)]

fn main() {
    unsafe {
        verona_rt_sys::enable_logging();
    }

    with_scheduler(|| {
        log(c"Hello World\n");

        // TODO: Less rigamarole
        let v = format!("{} + {} == {}\n", 2, 3, 2 + 3);
        let v = CString::new(v).unwrap();
        let v = Box::leak(Box::new(v));
        let v = v.as_c_str();

        log(v);
    });

    unsafe {
        verona_rt_sys::dump_flight_recorder();
    }
}
