use boxcars::Cown;

struct Foo;

impl Drop for Foo {
    fn drop(&mut self) {
        dbg!();
    }
}

fn main() {
    boxcars::with_scheduler(|| {
        dbg!();
        let foo = Cown::new(Foo);
        let f2 = foo.clone();
        dbg!();
        drop(f2);
        dbg!();
        drop(foo);
    })
}
