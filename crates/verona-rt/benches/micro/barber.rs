use boxcars::{when, with_scheduler, Cown};

use stdx::rand::SimpleRand;

struct CustomerFactory {
    number_of_haircuts: u64,
    attempts: u64,
    room: Cown<WaitingRoom>,
    random: SimpleRand,
}

impl CustomerFactory {
    fn new(number_of_haircuts: u64, room: Cown<WaitingRoom>) -> Self {
        Self {
            number_of_haircuts,
            attempts: 0,
            room,
            random: SimpleRand::default(),
        }
    }
}

struct Customer {
    factory: Cown<CustomerFactory>,
}
impl Customer {
    fn new(factory: Cown<CustomerFactory>) -> Self {
        Self { factory }
    }

    fn sit_down(&self) {}
}

struct Barber {
    haircut_rate: u32,
    random: SimpleRand,
}

impl Barber {
    fn new(haircut_rate: u32) -> Self {
        Self {
            haircut_rate,
            random: SimpleRand::default(),
        }
    }
}

fn barberwait(_wait: u32, _random: &mut SimpleRand) -> u32 {
    // let mut x = 0;
    // for _ in 0..wait {
    //     random.next();
    //     x += 1;
    // }
    // x
    0
}

struct WaitingRoom {
    size: u64,
    count: u64,
    barber: Cown<Barber>,
}

impl WaitingRoom {
    fn new(size: u64, barber: Cown<Barber>) -> Self {
        Self {
            size,
            barber,
            count: 0,
        }
    }

    fn enter(wr: &Cown<Self>, customer: Cown<Customer>) {
        when(wr, move |mut wr| {
            if wr.count == wr.size {
                Customer::full(&customer);
            } else {
                wr.count += 1;

                when((&wr.barber, &customer), {
                    let wr = wr.cown();
                    move |(mut barber, customer)| {
                        when(&wr, |mut wr| wr.count -= 1);

                        customer.sit_down();

                        let rate = barber.haircut_rate;
                        let wait_for = barber.random.next_int_with_max(rate) + 10;
                        barberwait(wait_for, &mut barber.random);
                        CustomerFactory::left(&customer.factory, customer.cown());

                        when((&barber.cown(), &wr), |(_barber, _wr)| {});
                    }
                });
            }
        })
    }
}

impl CustomerFactory {
    fn returned(this: &Cown<Self>, customer: Cown<Customer>) {
        when(this, |mut this| {
            this.attempts += 1;
            WaitingRoom::enter(&this.room, customer);
        });
    }

    fn left(this: &Cown<Self>, _customer: Cown<Customer>) {
        when(this, |mut this| {
            this.number_of_haircuts -= 1;
        })
    }
}

impl CustomerFactory {
    fn run(this: &Cown<CustomerFactory>, rate: u32) {
        when(this, {
            let tag = this.clone();

            move |mut this| {
                for _ in 0..this.number_of_haircuts {
                    this.attempts += 1;
                    WaitingRoom::enter(&this.room, Cown::new(Customer::new(tag.clone())));
                    barberwait(this.random.next_int_with_max(rate) + 10, &mut this.random);
                }
            }
        })
    }
}

impl Customer {
    fn full(this: &Cown<Self>) {
        when(this, |this| {
            CustomerFactory::returned(&this.factory, this.cown())
        });
    }
}

pub fn bench_barber(haircuts: u64, room: u64, production: u32, cut: u32) {
    with_scheduler(|| {
        let barber = Cown::new(Barber::new(cut));

        let wr = Cown::new(WaitingRoom::new(room, barber));

        let cf = Cown::new(CustomerFactory::new(haircuts, wr));

        CustomerFactory::run(&cf, production);
    });
}
