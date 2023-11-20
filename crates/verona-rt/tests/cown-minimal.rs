use verona_rt::cown::CownPtr;

#[test]
fn main() {
    verona_rt::scheduler::with_leak_detector(|| {
        let v1 = CownPtr::new(101);
        drop(v1);
    });
}
