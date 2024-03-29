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
using acquired_cown = verona::cpp::acquired_cown<DtorThunk>;
using ActualCown = verona::cpp::ActualCown<DtorThunk>;

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

// Ensure we're right about the definition of cown_ptr/acquired_cown.
static_assert(sizeof(cown_ptr) == sizeof(void*));
static_assert(sizeof(acquired_cown) == sizeof(void*));

static constexpr size_t actual_sz = sizeof(ActualCown);
static_assert(alignof(ActualCown) == alignof(void*));

// It's critical that you don't pass a cown_ptr or an actual_cown directly as an
// argument, or use them in a return type, as that'll lead to the wrong ABI.
//
// Instead, use pointer and out-params.
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
    *out = *in;
  }
  void boxcar_cownptr_drop(cown_ptr* ptr)
  {
    ptr->~cown_ptr();
  }
  void boxcar_cownptr_new(size_t size, void (*dtor)(void*), cown_ptr* out)
  {
    *out = verona::cpp::make_boxcar_cown(size, dtor);
  }
  void boxcar_acquiredcown_cown(acquired_cown* ptr, cown_ptr* out)
  {
    *out = ptr->cown();
  }

  void boxcar_size_info(
    size_t* sizeof_actualcown,
    size_t* alignof_actualcown,
    size_t* sizeof_object_header,
    size_t* object_alignment)
  {
    *sizeof_actualcown = sizeof(ActualCown);
    *alignof_actualcown = alignof(ActualCown);
    *sizeof_object_header = sizeof(verona::rt::Object::Header);
    *object_alignment = verona::rt::Object::ALIGNMENT;
  }

  void
  boxcar_when1(cown_ptr* cown, void (*func)(acquired_cown*, void*), void* data)
  {
    when(*cown) << [=](acquired_cown acq) { func(&acq, data); };
  }
  void boxcar_when2(
    cown_ptr* c1,
    cown_ptr* c2,
    void (*func)(acquired_cown*, acquired_cown*, void*),
    void* data)
  {
    when(*c1, *c2) << [=](auto a1, auto a2) { func(&a1, &a2, data); };
  }

  int32_t boxcars_add(int32_t a, int32_t b)
  {
    return a + b;
  }
}
