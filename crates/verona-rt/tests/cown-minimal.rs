use verona_rt::Cown;

#[test]
fn main() {
    unsafe {
        verona_rt_sys::enable_logging();
    }

    verona_rt::with_scheduler(|| {
        let v1 = Cown::new(101);
        drop(v1);
    });
}
