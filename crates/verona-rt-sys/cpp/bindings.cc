// std
#include <bit>
#include <cstdint>
#include <string_view>
// verona
#include <cpp/cown.h>
#include <cpp/when.h>
#include <sched/schedulerthread.h>

using verona::cpp::make_cown;
using verona::cpp::when;
using verona::rt::Scheduler;
using verona::rt::VCown;

using verona::cpp::DtorThunk;

using cown_ptr = verona::cpp::cown_ptr<DtorThunk>;
using aquired_cown = verona::cpp::acquired_cown<DtorThunk>;
using ActualCown = verona::cpp::ActualCown<DtorThunk>;

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

// Ensure we're right about the definition of cown_ptr/aquired_cown.
static_assert(sizeof(cown_ptr) == sizeof(void*));
static_assert(sizeof(aquired_cown) == sizeof(void*));

static constexpr size_t actual_sz = sizeof(ActualCown);
static constexpr size_t valloc_size = verona::rt::vsizeof<DtorThunk>;

static_assert(valloc_size == actual_sz);
static_assert(alignof(ActualCown) == alignof(void*));

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
  void dump_flight_recorder()
  {
    Logging::SysLog::dump_flight_recorder();
  }

  /*
   * Cown
   */
  void boxcar_cownptr_clone(cown_ptr* in, cown_ptr* out)
  {
    *in = *out;
  }
  void boxcar_cownptr_drop(cown_ptr* ptr)
  {
    // TODO: Run custom dtor.
    ptr->~cown_ptr();
  }
  void boxcar_cownptr_new(size_t size, void (*dtor)(void*), cown_ptr* out)
  {
    *out = verona::cpp::make_boxcar_cown(size, dtor);
  }
  void boxcar_aquiredcown_cown(aquired_cown* ptr, cown_ptr* out)
  {
    *out = ptr->cown();
  }

  int32_t boxcars_add(int32_t a, int32_t b)
  {
    return a + b;
  }
}
