use boc::Cown;

#[test]
fn main() {
    unsafe {
        boc_sys::enable_logging();
    }

    boc::with_scheduler(|| {
        let v1 = Cown::new(101);
        drop(v1);
    });
}
