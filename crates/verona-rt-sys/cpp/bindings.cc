// TODO: Include what you use.
// std
#include <array>
#include <float.h>
#include <ostream>
#include <stddef.h>
#include <stdint.h>
#include <string_view>
// verona
#include <cpp/when.h>
#include <debug/harness.h>
#include <verona.h>

using verona::rt::Behaviour;
using verona::rt::BehaviourCore;
using verona::rt::Cown;
using verona::rt::Descriptor;
using verona::rt::Object;
using verona::rt::Scheduler;
using verona::rt::Slot;
using verona::rt::Work;

using verona::cpp::acquired_cown;
using verona::cpp::cown_ptr;
using verona::cpp::make_cown;
using verona::cpp::when;

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

namespace bench
{
  uint32_t sequential_fib(uint32_t n)
  {
    if (n <= 1)
      return n;
    else
      return sequential_fib(n - 1) + sequential_fib(n - 2);
  }

  void parallel_fib_into(uint32_t n, cown_ptr<uint32_t> result)
  {
    if (n <= 4)
    {
      when(result) << [n](auto r) { *r = sequential_fib(n); };
    }
    else
    {
      auto f1 = make_cown<uint32_t>(0);
      parallel_fib_into(n - 1, f1);
      parallel_fib_into(n - 2, result);
      when(result, f1) << [](auto r, auto f) { *r += f; };
    }
  }

  void parallel_fib_into_carefull(uint32_t n, cown_ptr<uint32_t>& result)
  {
    if (n <= 4)
    {
      when(result) << [n](auto r) { *r = sequential_fib(n); };
    }
    else
    {
      auto f1 = make_cown<uint32_t>(0);
      parallel_fib_into_carefull(n - 1, f1);
      parallel_fib_into_carefull(n - 2, result);
      when(result, f1) << [](auto r, auto f) { *r += f; };
    }
  }

  // https://github.com/ic-slurp/verona-benchmarks/blob/4238a9e0217495af95caf361fe9829bb82cca581/util/random.h#L20C1-L34C3
  struct SimpleRand /*: public Random */
  {
    uint64_t value;

    SimpleRand(uint64_t x) : value(x) {}

    SimpleRand() : SimpleRand(0xDEADBEAF) {}

    uint64_t next()
    {
      return nextLong();
    }

    uint64_t nextLong()
    {
      return std::exchange(value, ((value * 1309) + 13849) & 65535);
    }

    uint32_t nextInt()
    {
      return nextInt(0);
    }

    uint32_t nextInt(uint32_t max)
    {
      return max == 0 ? uint32_t(nextLong()) : (uint32_t(nextLong()) % max);
    }

    double nextDouble()
    {
      return double(1.0 / (nextLong() + 1));
    }
  };

  // https://github.com/ic-slurp/verona-benchmarks/blob/4238a9e0217495af95caf361fe9829bb82cca581/savina/boc/concurrency/banking.h#L18
  namespace banking
  {

    using verona::cpp::acquired_cown;

    struct Account
    {
      double balance;

      Account(double balance) : balance(balance) {}

      void debit(double amount)
      {
        balance -= amount;
      }

      void credit(double amount)
      {
        balance += amount;
      }
    };

    struct Teller
    {
      uint64_t transactions;
      SimpleRand random;
      uint64_t completed;
      std::vector<cown_ptr<Account>> accounts;

      Teller(
        double initial_balance, uint64_t num_accounts, uint64_t transactions)
      : transactions(transactions), random(SimpleRand(123456)), completed(0)

      {
        for (uint64_t i = 0; i < num_accounts; i++)
        {
          accounts.emplace_back(make_cown<Account>(initial_balance));
        }
      }

      static void spawn_transactions(const cown_ptr<Teller>& self)
      {
        when(self) << [tag = self](acquired_cown<Teller> self) mutable {
          for (uint64_t i = 0; i < self->transactions; i++)
          {
            // Randomly pick source and destination account
            uint64_t source;
            uint64_t dest;

            do
            { // changed from actors to avoid deadlock from aliasing
              source = self->random.nextInt((self->accounts.size() / 10) * 8);
              dest = self->random.nextInt(self->accounts.size() - source);
            } while (source == dest);

            if (dest == 0)
              dest++;

            const cown_ptr<Account>& src = self->accounts[source];
            const cown_ptr<Account>& dst = self->accounts[dest];
            double amount = self->random.nextDouble() * 1000;

            when(src, dst) << [amount, tag](
                                acquired_cown<Account> src,
                                acquired_cown<Account> dst) mutable {
              src->debit(amount);
              dst->credit(amount);

              Teller::reply(tag);
            };
          }
        };
      }

      static void reply(const cown_ptr<Teller>& self)
      {
        when(self) << [](acquired_cown<Teller> self) mutable {
          self->completed++;
          if (self->completed == self->transactions)
          {
            return;
          }
        };
      }
    };

  }

  namespace barber
  {

    struct Customer;
    struct WaitingRoom;

    struct CustomerFactory
    {
      uint64_t number_of_haircuts;
      uint64_t attempts;
      cown_ptr<WaitingRoom> room;
      SimpleRand random;

      CustomerFactory(uint64_t number_of_haircuts, cown_ptr<WaitingRoom>&& room)
      : number_of_haircuts(number_of_haircuts),
        attempts(0),
        room(std::move(room))
      {}

      static void run(cown_ptr<CustomerFactory>&, uint32_t);
      static void
      returned(const cown_ptr<CustomerFactory>&, cown_ptr<Customer>);
      static void left(const cown_ptr<CustomerFactory>&, cown_ptr<Customer>);
    };

    struct Customer
    {
      cown_ptr<CustomerFactory> factory;

      Customer(cown_ptr<CustomerFactory> factory) : factory(std::move(factory))
      {}

      static void full(const cown_ptr<Customer>&);
      void wait();
      void sit_down() {};
    };

    struct Barber
    {
      uint32_t haircut_rate;
      SimpleRand random;

      Barber(uint32_t haircut_rate) : haircut_rate(haircut_rate) {}

      static void
      enter(const cown_ptr<Barber>&, cown_ptr<Customer>, cown_ptr<WaitingRoom>);
    };

    __attribute__((noinline)) static uint32_t
    cpp_barberwait(uint32_t wait, SimpleRand& random)
    {
      uint32_t x = 0;

      for (uint32_t i = 0; i < wait; ++i)
      {
        random.next();
        x++;
      }

      return x;
    }

    struct WaitingRoom
    {
      const uint64_t size;
      uint64_t count = 0;
      cown_ptr<Barber> barber;

      WaitingRoom(uint64_t size, const cown_ptr<Barber>& barber)
      : size(size), barber(barber)
      {}

      static void
      enter(const cown_ptr<WaitingRoom>& wr, cown_ptr<Customer> customer)
      {
        when(wr) << [customer =
                       std::move(customer)](acquired_cown<WaitingRoom> wr) {
          if (wr->count == wr->size)
          {
            Customer::full(std::move(customer));
          }
          else
          {
            wr->count++;

            when(wr->barber, customer) << [wr = wr.cown()](
                                            acquired_cown<Barber> barber,
                                            acquired_cown<Customer> customer) {
              when(wr) << [](acquired_cown<WaitingRoom> wr) { wr->count--; };

              customer->sit_down();
              cpp_barberwait(
                barber->random.nextInt(barber->haircut_rate) + 10,
                barber->random);
              CustomerFactory::left(customer->factory, customer.cown());

              when(barber.cown(), wr) << [](
                                           acquired_cown<Barber> barber,
                                           acquired_cown<WaitingRoom> wr) {};
            };
          }
        };
      }
    };

    void CustomerFactory::returned(
      const cown_ptr<CustomerFactory>& self, cown_ptr<Customer> customer)
    {
      when(self) <<
        [customer = std::move(customer)](acquired_cown<CustomerFactory> self) {
          self->attempts++;
          WaitingRoom::enter(self->room, std::move(customer));
        };
    }

    void CustomerFactory::left(
      const cown_ptr<CustomerFactory>& self, cown_ptr<Customer> customer)
    {
      when(self) <<
        [](acquired_cown<CustomerFactory> self) { self->number_of_haircuts--; };
    }

    void CustomerFactory::run(cown_ptr<CustomerFactory>& self, uint32_t rate)
    {
      when(self) << [tag = self, rate](acquired_cown<CustomerFactory> self) {
        for (uint64_t i = 0; i < self->number_of_haircuts; ++i)
        {
          self->attempts++;
          WaitingRoom::enter(self->room, make_cown<Customer>(tag));
          cpp_barberwait(self->random.nextInt(rate) + 10, self->random);
        }
      };
    }

    void Customer::full(const cown_ptr<Customer>& self)
    {
      when(self) << [](acquired_cown<Customer> self) {
        CustomerFactory::returned(self->factory, self.cown());
      };
    }

    void Customer::wait() {}
  }

  namespace philosopher
  {
    using std::move;

    struct Table
    {
      size_t still_eating;

      Table(uint64_t philosophers) : still_eating(philosophers) {}

      static void finished(const cown_ptr<Table>& self)
      {
        when(self) << [](acquired_cown<Table> self) { --(self->still_eating); };
      }

      ~Table()
      {
        assert(still_eating == 0);
      }
    };

    struct Fork
    {
      Fork() {}
    };

    struct Philosopher
    {
      uint64_t rounds;
      cown_ptr<Fork> left;
      cown_ptr<Fork> right;
      cown_ptr<Table> table;

      Philosopher(
        uint64_t rounds,
        cown_ptr<Fork> left,
        cown_ptr<Fork> right,
        cown_ptr<Table> table)
      : rounds(rounds), left(move(left)), right(move(right)), table(move(table))
      {}

      static void eat(cown_ptr<Philosopher> phil)
      {
        when(phil) << [](acquired_cown<Philosopher> phil) {
          phil->rounds--;
          if (phil->rounds > 0)
          {
            when(phil->left, phil->right)
              << [](acquired_cown<Fork> left, acquired_cown<Fork> right) {};
            eat(phil.cown());
          }
          else
          {
            Table::finished(phil->table);
          }
        };
      }
    };
  }
}

extern "C"
{
  /*
   * Scheduler
   */

  /// Returns a static global, so always safe AFAIKT.
  Scheduler* scheduler_get(void)
  {
    return &Scheduler::get();
  }
  void scheduler_init(Scheduler& sched, size_t count)
  {
    sched.init(count);
  }
  void scheduler_run(Scheduler* sched)
  {
    sched->run();
  }

  void schedular_set_detect_leaks(bool detect_leaks)
  {
    Scheduler::set_detect_leaks(detect_leaks);
  }

  static bool get_has_leaks()
  {
    bool is_ok = true;
#ifdef SNMALLOC_TRACING
    snmalloc::message<1024>("!! checking for leaks");
#endif

    snmalloc::debug_check_empty<snmalloc::Alloc::Config>(&is_ok);
#ifdef SNMALLOC_TRACING
    snmalloc::message<1024>("!! leak check done, is_ok={}", is_ok);
#endif

    return !is_ok;
  }

  bool schedular_has_leaks()
  {
    verona::rt::LocalEpochPool::sort();

    bool has_leaks = get_has_leaks();

    if (has_leaks)
    {
#ifdef SNMALLOC_TRACING
      snmalloc::message<1024>("!! Leaks detected, trying double jeopardy");
#endif

      // Double Jeopardy: See if we still have leaks after waiting
      // a short while for more destructors/gc to run on other threads.
      //
      // This is terrible practice to use sleep for sync, but in this case we've
      // already goofed, and it's usefull to know if the leaks are due to some
      // race condition here. Origionally added for #21, we'll see if it
      // remains.
      std::this_thread::sleep_for(std::chrono::milliseconds(10));
      if (!get_has_leaks())
      {
#ifdef SNMALLOC_TRACING
        snmalloc::message<1024>("!! Double jeopardy found leaks disapearing??");
#endif
        std::cerr << "??? leaks disapeared by magic???" << std::endl;

#ifdef USE_FLIGHT_RECORDER
        Logging::SysLog::dump_flight_recorder();
#endif

        abort();
      }
    }

    return has_leaks;
  }

  /*
   * Logging
   */
  void enable_logging()
  {
    Logging::enable_logging();
  }
  void boxcar_log_cstr(const char* str)
  {
    Logging::cout() << str;
  }
  void boxcar_log_endl()
  {
    Logging::cout() << std::endl;
  }
  void boxcar_log_usize(size_t v)
  {
    Logging::cout() << v;
  }
  void boxcar_log_ptr(void* p)
  {
    Logging::cout() << p;
  }
  void boxcars_dump_flight_recorder()
  {
    Logging::SysLog::dump_flight_recorder();
  }

  /*
   * Cown
   */
  void boxcars_acquire_object(Cown* o)
  {
    Cown::acquire(o);
  }
  void boxcars_release_object(Cown* o)
  {
    auto& alloc = verona::rt::ThreadAlloc::get();
    Cown::release(alloc, o);
  }

  void
  boxcar_vsizeof_info(size_t* sizeof_object_header, size_t* object_alignment)
  {
    *sizeof_object_header = sizeof(verona::rt::Object::Header);
    *object_alignment = verona::rt::Object::ALIGNMENT;
  }

  void boxcar_vsizeof_examples(
    size_t* boolsize, size_t* i32size, size_t* voidptrsize, size_t* charx17size)
  {
    *boolsize = verona::rt::vsizeof<bool>;
    *i32size = verona::rt::vsizeof<int32_t>;
    *voidptrsize = verona::rt::vsizeof<void*>;
    *charx17size = verona::rt::vsizeof<std::array<char, 17>>;
  }

  Cown* boxcars_allocate_cown(Descriptor* desc)
  {
    size_t size = desc->size;
    void* base = snmalloc::ThreadAlloc::get().alloc(size);

    Object* obj = Object::register_object(base, desc);

    Cown* cown = new (obj) Cown();

    Logging::cout() << "Registeded cown at address " << cown << Logging::endl;

    return cown;
  }

  void boxcars_sched_lambda(
    size_t n_cowns,
    Cown** cowns,
    void (*f)(Work*),
    size_t payload_size,
    void* payload)
  {
    /* static Behaviour* make(size_t count, T&& f) */
    auto* behaviour_core = BehaviourCore::make(n_cowns, f, payload_size);
    memcpy(behaviour_core->get_body(), payload, payload_size);

    /* prepare_to_schedule(size_t count, Request* requests, T&& f) */
    auto* body = (Behaviour*)(behaviour_core);
    auto* slots = body->get_slots();
    for (size_t i = 0; i < n_cowns; i++)
    {
      new (&slots[i]) verona::rt::Slot(cowns[i]);
      // TODO: We can put move info here.
    }

    /* schedule(size_t count, Request* requests, T&& f) */
    BehaviourCore* arr[] = {body};
    BehaviourCore::schedule_many(arr, 1);
  }

  /*
   * Helpers to implement Behaviour::invoke
   */
  void
  boxcars_preinvoke(Work* work, Slot** slots, void** body, size_t* count_out)
  {
    auto* be = BehaviourCore::from_work(work);
    *slots = be->get_slots();
    *body = be->get_body();
    *count_out = be->count;
  }
  void boxcars_postinvoke(Work* work)
  {
    auto* be = BehaviourCore::from_work(work);
    be->release_all();
    work->dealloc();
  }

  void boxcars_test_descriptor_info(size_t* size, size_t* align)
  {
    *size = sizeof(Descriptor);
    *align = alignof(Descriptor);
  }
  void boxcars_test_cown_info(size_t* size, size_t* align)
  {
    *size = sizeof(Cown);
    *align = alignof(Cown);
  }
  void boxcars_test_slot_info(size_t* size, size_t* align)
  {
    *size = sizeof(Slot);
    *align = alignof(Slot);
  }

  void boxcars_snmalloc_message(const char* ptr, size_t len)
  {
    std::string_view s(ptr, len);
#ifdef SNMALLOC_TRACING
    snmalloc::message<1024>("{}", s);
#endif
  }

  void boxcars_busy_loop(size_t usecs)
  {
    busy_loop(usecs);
  }

  // TODO: Don't put these in main binary
  void bbench_create_n_cowns(size_t n)
  {
    std::vector<cown_ptr<size_t>> v;
    v.reserve(n);

    for (size_t i = 0; i < n; i++)
    {
      v.push_back(make_cown<size_t>(i));
    }
  }

  void bbench_busyloop_inside_when(size_t nsecs, uint64_t iters)
  {
    Scheduler::get().init(1);

    auto c = make_cown<size_t>(nsecs);

    for (int i = 0; i < iters; i++)
    {
      when(c) << [](auto c) { busy_loop(*c); };
    }

    Scheduler::get().run();
  }

  void bbench_schedule_n_lambdas_onto_cown(size_t n, uint64_t iters)
  {
    Scheduler::get().init(1);

    auto threader = make_cown<int>(0);

    for (int i = 0; i < iters; i++)
    {
      auto c = make_cown<int>(0);
      for (int j = 0; j < n; j++)
      {
        when(c) << [](auto c) { c++; };
      }

      when(c, threader) << [](auto, auto) {};
    }

    Scheduler::get().run();
  }

  void bbench_do_par_fib(uint32_t n, uint32_t exp, uint64_t iters)
  {
    Scheduler::get().init(1);

    auto r = make_cown<uint32_t>(0);

    for (int i = 0; i < iters; i++)
    {
      bench::parallel_fib_into(n, r);
      when(r) << [exp](auto r) {
        if (r != exp)
          abort();
      };
    }

    Scheduler::get().run();
  }

  void bbench_do_par_fib_carefull(uint32_t n, uint32_t exp, uint64_t iters)
  {
    Scheduler::get().init(1);

    auto r = make_cown<uint32_t>(0);

    for (int i = 0; i < iters; i++)
    {
      bench::parallel_fib_into_carefull(n, r);
      when(r) << [exp](auto r) {
        if (r != exp)
          abort();
      };
    }

    Scheduler::get().run();
  }

  void bbench_do_banking(uint64_t acccounts, uint64_t transactions)
  {
    Scheduler::get().init(1);
    double initial = DBL_MAX / float(acccounts * transactions);

    auto teller =
      make_cown<bench::banking::Teller>(initial, acccounts, transactions);

    bench::banking::Teller::spawn_transactions(teller);

    Scheduler::get().run();
  }

  void bbench_do_barber(
    uint64_t haircuts, uint64_t room, uint32_t production, uint32_t cut)
  {
    Scheduler::get().init(1);

    using namespace bench::barber;

    auto barber = make_cown<Barber>(cut);
    auto wr = make_cown<WaitingRoom>(room, std::move(barber));
    auto cf = make_cown<CustomerFactory>(haircuts, std::move(wr));

    CustomerFactory::run(cf, production);

    Scheduler::get().run();
  }

  void
  bbench_do_philosopher(size_t philosophers, uint64_t rounds, size_t n_threads)
  {
    using namespace bench::philosopher;

    Scheduler::get().init(n_threads);

    cown_ptr<Table> table = make_cown<Table>(philosophers);

    cown_ptr<Fork> first = make_cown<Fork>();
    cown_ptr<Fork> prev = first;
    for (size_t i = 0; i < philosophers - 1; ++i)
    {
      cown_ptr<Fork> next = make_cown<Fork>();
      Philosopher::eat(make_cown<Philosopher>(rounds, move(prev), next, table));
      prev = move(next);
    }
    Philosopher::eat(
      make_cown<Philosopher>(rounds, move(prev), move(first), move(table)));

    Scheduler::get().run();
  }

  void bbench_create_scheduler(void)
  {
    Scheduler::get().init(1);
    Scheduler::get().run();
  }
}
