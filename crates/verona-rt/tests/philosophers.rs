use verona_rt::{when, with_scheduler, Cown};

struct Table {
    done_eating: usize,
}

impl Table {
    fn finished(this: Cown<Self>) {
        when(&this, |mut this| {
            this.done_eating -= 1;
        })
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        debug_assert_eq!(self.done_eating, 0);
    }
}

#[derive(Default)]
struct Fork {}

struct Philosopher {
    rounds: u64,
    left: Cown<Fork>,
    right: Cown<Fork>,
    table: Cown<Table>,
}

impl Philosopher {
    fn eat(this: &Cown<Self>) {
        when(this, |mut this| {
            this.rounds -= 1;

            if this.rounds > 0 {
                when((&this.left, &this.right), |(_, _)| {});
                Philosopher::eat(&this.cown());
            } else {
                Table::finished(this.table.clone());
            }
        })
    }
}

pub fn do_phil(philosophers: usize, rounds: u64, optimal: bool) {
    with_scheduler(|| {
        let table = Cown::new(Table {
            done_eating: philosophers,
        });

        let first = Cown::new(Fork::default());
        let mut prev = first.clone();

        let mut ps = Vec::new();

        for i in 0..(philosophers - 1) {
            let next = Cown::new(Fork::default());
            let p = Cown::new(Philosopher {
                rounds,
                left: prev,
                right: next.clone(),
                table: table.clone(),
            });
            ps.push(p);
            prev = next;
        }

        ps.push(Cown::new(Philosopher {
            rounds,
            left: prev,
            right: first,
            table,
        }));

        if optimal {
            for i in (0..philosophers).step_by(2) {
                Philosopher::eat(ps[i]);
            }
            for i in (1..philosophers).step_by(2) {
                Philosopher::eat(ps[i]);
            }
        } else {
            for p in &ps {
                Philosopher::eat(p);
            }
        }
    });
}

#[test]
fn short() {
    do_phil(2, 2);
}

#[test]
fn long() {
    do_phil(500, 1000);
}
