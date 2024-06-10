use verona_rt::{when, with_scheduler, Cown};

struct Pholosopher {
    rounds: u64,
    left: Cown<Fork>,
    right: Cown<Fork>,
    table: Cown<Table>,
}

#[derive(Default)]
struct Fork {}
#[derive(Default)]
struct Table {}

impl Pholosopher {
    fn eat(this: &Cown<Self>) {
        when(this, |mut this| {
            this.rounds -= 1;
            if this.rounds == 0 {
                Table::finished(&this.table);
            } else {
                when((&this.left, &this.right), {
                    let tag = this.cown();
                    move |(_, _)| {
                        Pholosopher::eat(&tag);
                    }
                });
            }
        })
    }
}

impl Table {
    fn finished(this: &Cown<Self>) {
        when(this, |_| {})
    }
}

pub fn do_phil(philosophers: u64, rounds: u64) {
    with_scheduler(|| {
        let table = Cown::new(Table::default());

        let first = Cown::new(Fork::default());
        let mut prev = first.clone();

        for _ in 0..philosophers {
            let next = Cown::new(Fork::default());
            let p = Cown::new(Pholosopher {
                rounds,
                left: prev,
                right: next.clone(),
                table: table.clone(),
            });
            Pholosopher::eat(&p);
            prev = next;
        }

        let p = Cown::new(Pholosopher {
            rounds,
            left: prev,
            right: first,
            table,
        });
        Pholosopher::eat(&p);
    });
}
