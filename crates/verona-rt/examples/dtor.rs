use verona_rt::Cown;

struct Foo;

impl Drop for Foo {
    fn drop(&mut self) {
        dbg!();
    }
}

fn main() {
    verona_rt::with_scheduler(|| {
        dbg!();
        let foo = Cown::new(Foo);
        let f2 = foo.clone();
        dbg!();
        drop(f2);
        dbg!();
        drop(foo);
    })
}
