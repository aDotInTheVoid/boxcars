use std::ffi::CString;

use boc::{log, with_scheduler};

// cargo test --test dump-flight-recorder --features flight_recorder

#[test]
#[cfg_attr(
    feature = "sanitizer_address",
    ignore = "we intentionally leak memory here"
)]

fn main() {
    unsafe {
        boc_sys::enable_logging();
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
        boc_sys::boxcars_dump_flight_recorder();
    }
}
