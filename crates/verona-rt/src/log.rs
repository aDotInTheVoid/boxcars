// TODO: Richer logging abstractions.

pub fn log_snmalloc(v: &str) {
    unsafe {
        verona_rt_sys::boxcars_snmalloc_message(v.as_ptr(), v.len());
    }
}

pub fn log(val: &'static core::ffi::CStr) {
    // TODO: Does this race?
    unsafe {
        verona_rt_sys::boxcar_log_cstr(val.as_ptr());
        verona_rt_sys::boxcar_log_endl();
    }
}

/*
cargo run --example log-around-dealloc --features systematic_testing

a TOP
a Registeded cown at address 0
a Just allocated
a Shared 0 acquire
a Just cloned
a Shared 0 release
a decref_shared 0x7f4a88510020
a dropped v1
a Shared 0 release
a decref_shared 0x7f4a88510020
a decref_shared part 20x7f4a88510020
a Cown 0 dealloc
a Collecting cown 0
a Cown 0 weak release
a Cown 0 no references left.
a dropped v2
*/
