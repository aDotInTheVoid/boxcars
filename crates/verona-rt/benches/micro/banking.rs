use boxcars::{when, Cown};

use stdx::rand::SimpleRand;

struct Account {
    balance: f64,
}

impl Account {
    fn new(balance: f64) -> Self {
        Self { balance }
    }
    fn debit(&mut self, amount: f64) {
        self.balance -= amount;
    }
    fn credit(&mut self, amount: f64) {
        self.balance += amount;
    }
}

pub(crate) struct Teller {
    transactions: u64,
    random: SimpleRand,
    completed: u64,
    accounts: Vec<Cown<Account>>,
}

impl Teller {
    pub fn new(initial_balance: f64, num_accounts: u64, transactions: u64) -> Teller {
        let mut accounts = Vec::new();

        for _ in 0..num_accounts {
            accounts.push(Cown::new(Account::new(initial_balance)));
        }

        Self {
            transactions,
            completed: 0,
            accounts,
            random: SimpleRand::new(123456),
        }
    }

    pub fn spawn_transactions(c: &Cown<Self>) {
        let tag = c.clone();
        when(c, move |mut this| {
            for _ in 0..this.transactions {
                let mut source;
                let mut dest;

                let n_accounts = this.accounts.len();

                loop {
                    source = this
                        .random
                        .next_int_with_max(((n_accounts / 10) * 8) as u32)
                        as usize;
                    dest = this.random.next_int_with_max((n_accounts - source) as u32) as usize;

                    if source != dest {
                        break;
                    }
                }

                let amount = this.random.next_double() * 1000.0;

                let src = &this.accounts[source];
                let dst = &this.accounts[dest];

                let tag = tag.clone();

                when((src, dst), move |(mut src, mut dst)| {
                    src.credit(amount);
                    dst.debit(amount);
                    Teller::reply(&tag);
                });
            }
        })
    }

    fn reply(c: &Cown<Teller>) {
        when(c, |mut this| {
            this.completed += 1;
        })
    }
}
