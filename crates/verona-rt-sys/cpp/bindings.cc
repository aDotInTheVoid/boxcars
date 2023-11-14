// std
#include <bit>
#include <cstdint>
// verona
#include <cpp/cown.h>
#include <sched/schedulerthread.h>

using verona::cpp::acquired_cown;
using verona::cpp::cown_ptr;
using verona::cpp::make_cown;
// using verona::cpp::when;
using verona::rt::Scheduler;
using verona::rt::VCown;

// struct Account : public VCown<Account>
// {
//   // Need explicit ctor for make_cown template magic.
//   Account(int balance, bool frozen) : balance_(balance), frozen_(frozen) {}

//   int balance_;
//   bool frozen_;
// };

// typedef void(use_account(acquired_cown<Account>&));
// typedef void(use_accout2(acquired_cown<Account>&, acquired_cown<Account>&));

// template<class T>
// void* cown_ptr_crimes(cown_ptr<T>* cown_ptr_ref)
// {
//   void** foo = std::bit_cast<void**, cown_ptr<T>*>(cown_ptr_ref);
//   return *foo;
// }

extern "C"
{
  /*
   * Scheduler
   */

  /// Returns a static global, so always safe AFAIKT.
  Scheduler& scheduler_get(void)
  {
    return Scheduler::get();
  }
  void scheduler_init(Scheduler& sched, size_t count)
  {
    sched.init(count);
  }
  void scheduler_run(Scheduler& sched)
  {
    sched.run();
  }

  void schedular_set_detect_leaks(bool detect_leaks)
  {
    Scheduler::set_detect_leaks(detect_leaks);
  }
  bool schedular_has_leaks()
  {
    bool result;
    snmalloc::debug_check_empty<snmalloc::Alloc::Config>(&result);
    return result;
  }

  /*
   * Cown
   */

  void cown_int_new(int32_t value, cown_ptr<int32_t>* out)
  {
    static_assert(sizeof(cown_ptr<int32_t>) == sizeof(void*));
    *out = make_cown<int32_t>(value);
  }

  void cown_int_delete(cown_ptr<int32_t>* value)
  {
    value->~cown_ptr();
  }

  void cown_int_clone(const cown_ptr<int32_t>& in, cown_ptr<int32_t>& out)
  {
    out = in;
  }

  // void when_account(cown_ptr<Account> account, use_account func)
  // {
  //   when(account) << [func](auto acc) { func(acc); };
  // }

  // void
  // when_account2(cown_ptr<Account> a1, cown_ptr<Account> a2, use_accout2 func)
  // {
  //   when(a1, a2) << [=](auto a1, auto a2) { func(a1, a2); };
  // }

  // cown_ptr<Account> cown(acquired_cown<Account> acc)
  // {
  //   return acc.cown();
  // }

  int32_t boxcars_add(int32_t a, int32_t b)
  {
    return a + b;
  }
}
