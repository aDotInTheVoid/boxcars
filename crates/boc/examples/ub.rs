use std::ops::DerefMut;

use boc::*;

fn main() {
    boc::with_scheduler(|| {
        let c = Cown::new(10);
        when((&c, &c), |(mut c1, mut c2)| {
            let ptr_1: &mut i32 = c1.deref_mut();
            let ptr_2: &mut i32 = c2.deref_mut();
            // aliasing mutable references, UB!!
        });
    })
}
