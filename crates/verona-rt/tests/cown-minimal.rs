use verona_rt::CownPtr;

#[test]
fn main() {
    unsafe {
        verona_rt_sys::enable_logging();
    }

    verona_rt::with_scheduler(|| {
        let v1 = CownPtr::new(101);
        drop(v1);
    });
}
