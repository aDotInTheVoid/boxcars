// std
#include <bit>
#include <cstdint>
#include <string_view>
// verona
#include <cpp/cown.h>
#include <cpp/when.h>
#include <sched/schedulerthread.h>

using verona::cpp::acquired_cown;
using verona::cpp::cown_ptr;
using verona::cpp::make_cown;
using verona::cpp::when;
using verona::rt::Scheduler;
using verona::rt::VCown;

// struct Account : public VCown<Account>
// {
//   // Need explicit ctor for make_cown template magic.
//   Account(int balance, bool frozen) : balance_(balance), frozen_(frozen) {}

//   int balance_;
//   bool frozen_;
// };

typedef void(use_int(acquired_cown<int32_t>& cown, void* data));
// typedef void(use_accout2(acquired_cown<Account>&, acquired_cown<Account>&));

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
    bool is_ok;
    snmalloc::debug_check_empty<snmalloc::Alloc::Config>(&is_ok);
    return !is_ok;
  }

  void enable_logging()
  {
    Logging::enable_logging();
  }

  void boxcar_log(const char* str, size_t len)
  {
    Logging::cout() << std::string_view(str, len) << Logging::endl;
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

  void cown_int_when1(const cown_ptr<int32_t>& cown, use_int func, void* data)
  {
    when(cown) << [=](auto cown) { func(cown, data); };
  }
  int32_t& cown_get_ref(acquired_cown<int32_t> const& cown)
  {
    return cown.get_ref();
  }
  void cown_get_cown(acquired_cown<int32_t> const& cown, cown_ptr<int32_t>& out)
  {
    out = cown.cown();
  }

  int32_t boxcars_add(int32_t a, int32_t b)
  {
    return a + b;
  }
}
