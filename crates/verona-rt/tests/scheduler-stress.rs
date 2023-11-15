use std::thread;

use verona_rt::scheduler::with;

#[test]
#[ignore = "https://github.com/aDotInTheVoid/boxcars/issues/4"]
fn stress() {
    for _ in 0..10 {
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    for _ in 0..10 {
                        with(|| {})
                    }
                });
            }
            s.spawn(|| {
                let r = std::panic::catch_unwind(|| with(|| panic!("lol lmao")));
                let r_err = r.unwrap_err();
                let s = r_err.downcast::<&str>().unwrap();
                assert_eq!(&**s, "lol lmao");
            });
        })
    }
}
