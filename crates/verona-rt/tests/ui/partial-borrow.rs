use verona_rt::{when, with_scheduler, AcquiredCown, Cown};

use std::ops::DerefMut;

struct Foo {
    a: i32,
    b: i32,
}

fn use_ints(x: &mut i32, y: &mut i32) {
    *x += 1;
    *y += 1;
}

fn main() {
    with_scheduler(|| {
        let cown = Cown::new(Foo { a: 1, b: 1 });
        when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
            use_ints(&mut acq_cown.a, &mut acq_cown.b)
        });

        when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
            let mut_ref: &mut Foo = &mut *acq_cown;
            use_ints(&mut mut_ref.a, &mut mut_ref.b)
        });

        when(&cown, |mut acq_cown: AcquiredCown<Foo>| {
            use_ints(
                &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).a,
                &mut <AcquiredCown<Foo> as DerefMut>::deref_mut(&mut acq_cown).b,
            )
        })
    });

    // This works
    let mut foo = Foo { a: 1, b: 1 };
    use_ints(&mut foo.a, &mut foo.b);

    // This works
    with_scheduler(|| {
        fn print_str(s: &str) {
            println!("{s}");
        }

        let cown = Cown::<&str>::new("hello world");
        when(&cown, |acq_cown: AcquiredCown<&str>| {
            print_str(&acq_cown);
        });
    });
}
