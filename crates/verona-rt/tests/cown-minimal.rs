use verona_rt::CownPtr;

#[test]
fn main() {
    verona_rt::with_scheduler(|| {
        let v1 = CownPtr::new(101);
        drop(v1);
    });
}
