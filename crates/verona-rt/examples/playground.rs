use boxcars::{when, with_scheduler, Cown};

fn borrow_from_stack() {
    let mut my_vec = vec![1, 2, 3];
    let vec_ref: &mut Vec<i32> = &mut my_vec;
    let cown = Cown::new(my_vec);
    vec_ref.push(4);
}

fn main() {
    with_scheduler(|| {
        borrow_from_stack();
    })
}
