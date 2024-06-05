use std::ops;

use crate::{when, with_scheduler, CownPtr};

struct Account {
    balance: u64,
    frozen: bool,
}

impl Account {
    fn create() -> Self {
        Self {
            balance: 1000,
            frozen: false,
        }
    }
}

#[test]
#[allow(unused)]
fn list1() {
    let mut acc1 = Account::create();

    acc1.balance -= 100;

    let mut acc2 = CownPtr::new(Account::create());
    /*
    error[E0609]: no field `balance` on type `CownPtr<Account>`
      --> crates/verona-rt/examples/list-1.rs:18:10
       |
    18 |     acc2.balance -= 100;
       |          ^^^^^^^ unknown field
     */
    // acc2.balance -= 100;
}

#[test]
fn list2() {
    fn transfer(src: &CownPtr<Account>, dst: &CownPtr<Account>, amount: u64) {
        when(src, move |mut src| src.balance -= amount);
        when(dst, move |mut dst| dst.balance += amount);
    }

    with_scheduler(|| {
        let a1 = CownPtr::new(Account::create());
        let a2 = CownPtr::new(Account::create());

        transfer(&a1, &a2, 100);

        when((&a1, &a2), |(a1, a2)| {
            assert_eq!(a1.balance, 900);
            assert_eq!(a2.balance, 1100);
        })
    });
}

#[test]
fn list3() {
    fn transfer(src: CownPtr<Account>, dst: CownPtr<Account>, amount: u64) {
        when(&src, move |mut src| {
            if src.balance >= amount {
                src.balance -= amount;
                when(&dst, move |mut dst| dst.balance += amount)
            }
        });
    }

    with_scheduler(|| {
        let src = CownPtr::new(Account::create());
        let dst = CownPtr::new(Account::create());

        transfer(src.clone(), dst.clone(), 100);

        when((&src, &dst), |(src, dst)| {
            // This get's scheduled after the removal, but before the deposit.
            // eprintln!("a1={src:?}, a2={dst:?}");
            assert_eq!(src.balance, 900);
            assert_eq!(dst.balance, 1000);
        })
    });
}

#[test]
fn list4() {
    fn transfer(src: CownPtr<Account>, dst: CownPtr<Account>, amount: u64) {
        when(&src, move |mut src| {
            if src.balance >= amount {
                src.balance -= amount;
                when(&dst, move |mut dst| dst.balance += amount)
            }
        });
    }

    with_scheduler(|| {
        let src = CownPtr::new(Account::create());
        let dst = CownPtr::new(Account::create());

        transfer(src.clone(), dst.clone(), 1);
        transfer(dst.clone(), src.clone(), 2);

        when((&src, &dst), |(src, dst)| {
            assert_eq!(src.balance, 999);
            assert_eq!(dst.balance, 998);
        })
    });
}

#[test]
fn list5() {
    fn transfer(src: &CownPtr<Account>, dst: &CownPtr<Account>, amount: u64) {
        when((src, dst), move |(mut src, mut dst)| {
            if src.balance >= amount && !src.frozen && !dst.frozen {
                src.balance -= amount;
                dst.balance += amount;
            }
        })
    }

    with_scheduler(|| {
        let src = CownPtr::new(Account::create());
        let dst = CownPtr::new(Account::create());
        transfer(&src, &dst, 100);

        when((&src, &dst), |(src, dst)| {
            assert_eq!(src.balance, 900);
            assert_eq!(dst.balance, 1100);
        });
    });

    with_scheduler(|| {
        let src = CownPtr::new(Account::create());
        let dst = CownPtr::new(Account::create());

        when(&dst, |mut dst| dst.frozen = true);

        transfer(&src, &dst, 100);

        when((&src, &dst), |(src, dst)| {
            assert_eq!(src.balance, 1000);
            assert_eq!(dst.balance, 1000);
        });
    })
}

#[test]
fn list6() {
    fn transfer(src: &CownPtr<Account>, dst: &CownPtr<Account>, amount: u64) {
        when((src, dst), move |(mut src, mut dst)| {
            if src.balance >= amount && !src.frozen && !dst.frozen {
                src.balance -= amount;
                dst.balance += amount;
            }
        })
    }

    with_scheduler(|| {
        let s1 = CownPtr::new(Account::create());
        let s2 = CownPtr::new(Account::create());
        let s4 = CownPtr::new(Account::create());

        transfer(&s1, &s2, 10);
        transfer(&s2, &s4, 20);

        when((&s1, &s2, &s4), |(s1, s2, s4)| {
            assert_eq!(s1.balance, 990);
            assert_eq!(s2.balance, 990);
            assert_eq!(s4.balance, 1020);
        });
    });
}

#[test]
fn list7() {
    fn transfer(src: &CownPtr<Account>, dst: &CownPtr<Account>, amount: u64) {
        when((src, dst), move |(mut src, mut dst)| {
            if src.balance >= amount && !src.frozen && !dst.frozen {
                src.balance -= amount;
                dst.balance += amount;
            }
        })
    }

    with_scheduler(|| {
        let s1 = CownPtr::new(Account::create());
        let s2 = CownPtr::new(Account::create());
        let s3 = CownPtr::new(Account::create());
        let s4 = CownPtr::new(Account::create());

        transfer(&s1, &s2, 10);
        transfer(&s3, &s4, 20);

        when((&s1, &s2, &s3, &s4), |(s1, s2, s3, s4)| {
            assert_eq!(s1.balance, 990);
            assert_eq!(s2.balance, 1010);
            assert_eq!(s3.balance, 980);
            assert_eq!(s4.balance, 1020);
        });
    });
}

#[test]
fn list8() {
    let (tx, rx) = std::sync::mpsc::channel();

    #[allow(unused_must_use)]
    with_scheduler(|| {
        let src = CownPtr::new(());
        let dst = CownPtr::new(());

        let log = CownPtr::new(tx);

        when(&log, |log| {
            log.send("begin");
        });
        let log_ = log.clone();
        when(&src, move |_| {
            when(&log_, |log| {
                log.send("deposit");
            })
        });
        let log_ = log.clone();
        when(&dst, move |_| {
            when(&log_, |log| {
                log.send("freeze");
            })
        });
        let log_ = log.clone();
        when((&src, &dst), move |_| {
            when(&log_, |log| {
                log.send("transfer");
            })
        });
    });

    let l1 = rx.recv().unwrap();
    let l2 = rx.recv().unwrap();
    let l3 = rx.recv().unwrap();
    let l4 = rx.recv().unwrap();

    match (l1, l2, l3, l4) {
        ("begin", "deposit", "freeze", "transfer") => {}
        ("begin", "freeze", "deposit", "transfer") => {}
        other => panic!("invalid order {other:?}"),
    }
}

#[test]
fn dining_philosophers() {
    const HUNGER: usize = 500;
    const NUM_PHILOSOPHERS: usize = 50;

    #[derive(Default)]
    struct Fork {
        uses: usize,
    }

    impl ops::Drop for Fork {
        fn drop(&mut self) {
            assert_eq!(self.uses, HUNGER * 2);
        }
    }
    fn get_left(forks: &[CownPtr<Fork>], index: usize) -> CownPtr<Fork> {
        forks[index].clone()
    }
    fn get_right(forks: &[CownPtr<Fork>], index: usize) -> CownPtr<Fork> {
        forks[(index + 1) % NUM_PHILOSOPHERS].clone()
    }
    struct Philosopher {
        left: CownPtr<Fork>,
        right: CownPtr<Fork>,
        hunger: usize,
    }
    impl Philosopher {
        fn new(forks: &[CownPtr<Fork>], index: usize) -> Self {
            Self {
                left: get_left(forks, index),
                right: get_right(forks, index),
                hunger: HUNGER,
            }
        }

        fn eat(mut self) {
            if self.hunger > 0 {
                when(
                    (&self.left.clone(), &self.right.clone()),
                    move |(mut f1, mut f2)| {
                        f1.uses += 1;
                        f2.uses += 1;
                        self.hunger -= 1;
                        self.eat();
                    },
                )
            }
        }
    }

    with_scheduler(|| {
        let mut forks = Vec::new();
        for _ in 0..NUM_PHILOSOPHERS {
            forks.push(CownPtr::new(Fork::default()));
        }
        for i in 0..NUM_PHILOSOPHERS {
            let p = Philosopher::new(&forks, i);
            p.eat()
        }
    });
}
