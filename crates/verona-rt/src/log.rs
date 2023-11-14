pub fn log(val: &str) {
    // TODO: Does this race?
    unsafe { verona_rt_sys::boxcar_log(val.as_ptr(), val.len()) }
}
