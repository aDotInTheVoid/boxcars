use std::ffi::CString;

use verona_rt::{log::log, scheduler};

// cargo test --test dump-flight-recorder --features flight_recorder

#[test]
fn main() {
    unsafe {
        verona_rt_sys::enable_logging();
    }

    scheduler::with(|| {
        log(cstr::cstr!("Hello World\n"));

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
