// TODO: Richer logging abstractions.

pub fn log(val: &'static std::ffi::CStr) {
    // TODO: Does this race?
    unsafe { verona_rt_sys::boxcar_log_cstr(val.as_ptr()) }
}
