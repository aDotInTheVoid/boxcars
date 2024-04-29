// TODO: Include what you use.
// std
#include <array>
#include <ostream>
#include <stddef.h>
#include <stdint.h>
// verona
#include <cpp/lambdabehaviour.h>
#include <object/object.h>
#include <sched/cown.h>
#include <sched/schedulerthread.h>

using verona::rt::Cown;
using verona::rt::Descriptor;
using verona::rt::Object;
using verona::rt::Scheduler;

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

typedef void (*WhenNFunc)(Cown**, size_t, void*);
typedef void (*When1Func)(Cown*, void*);
typedef void (*When2Func)(Cown*, Cown*, void*);

template<size_t N_COWNS>
struct CownThunk
{
  std::array<Cown*, N_COWNS> cowns_;
  WhenNFunc thunk_;
  void* data_;

  void operator()()
  {
    thunk_(cowns_.data(), N_COWNS, data_);
  }
};

template<size_t N_COWNS>
static std::array<Cown*, N_COWNS> gather_cown(size_t len, Cown** ptr)
{
  assert(len == N_COWNS);
  std::array<Cown*, N_COWNS> arr;
  memcpy(arr.data(), ptr, sizeof(arr));
  return arr;
}

template<size_t N_COWNS>
static void schedule_n(size_t len, Cown** ptr, WhenNFunc func, void* data)
{
  auto cownarr = gather_cown<N_COWNS>(len, ptr);
  auto lambda = CownThunk<N_COWNS>{cownarr, func, data};
  verona::rt::schedule_lambda(len, ptr, std::move(lambda));
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
  bool schedular_has_leaks()
  {
    bool is_ok = true;
    snmalloc::debug_check_empty<snmalloc::Alloc::Config>(&is_ok);
    // snmalloc::debug_check_empty<snmalloc::Alloc::Config>();
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

  // TODO: Use requests
  // TODO: Variadic.
  void boxcars_schedule_1(Cown* cown, When1Func func, void* data)
  {
    verona::rt::schedule_lambda(cown, [=]() { func(cown, data); });
  }
  void boxcars_schedule_2(Cown* c1, Cown* c2, When2Func func, void* data)
  {
    Cown* cowns[2] = {c1, c2};
    verona::rt::schedule_lambda(2, cowns, [=]() { func(c1, c2, data); });
  }

#define BUILD_SCHEDULE(n) \
  void boxcars_sched_##n(size_t len, Cown** ptr, WhenNFunc func, void* data) \
  { \
    schedule_n<n>(len, ptr, func, data); \
  }

  BUILD_SCHEDULE(1)
  BUILD_SCHEDULE(2)
  BUILD_SCHEDULE(3)
  BUILD_SCHEDULE(4)
  BUILD_SCHEDULE(5)
  BUILD_SCHEDULE(6)
  BUILD_SCHEDULE(7)
  BUILD_SCHEDULE(8)
  BUILD_SCHEDULE(9)
  BUILD_SCHEDULE(10)

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
}
