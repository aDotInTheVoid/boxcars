// TODO: Richer logging abstractions.

pub fn log(val: &'static std::ffi::CStr) {
    // TODO: Does this race?
    unsafe {
        verona_rt_sys::boxcar_log_cstr(val.as_ptr());
        verona_rt_sys::boxcar_log_endl();
    }
}

/*
cargo run --example log-around-dealloc --features systematic_testing

a TOP
a Just alloced
a Shared 0 acquire
a Just cloned
a Shared 0 release
a decref_shared 0x7f9e090a0020
a droped v1
a Shared 0 release
a decref_shared 0x7f9e090a0020
a decref_shared part 2 0x7f9e090a0020
a Cown 0 dealloc
a Collecting cown 0
a Cown 0 weak release
a Cown 0 no references left.
a droped v2
*/
