use verona_rt::Cown;

fn main() {
    struct Foo {
        number: i32,
        string: &'static str,
    }

    impl Foo {
        fn new(number: i32, string: &'static str) -> Self {
            Self { number, string }
        }
    }

    let c_foo = Cown::new(Foo::new("hello", 101));
}
