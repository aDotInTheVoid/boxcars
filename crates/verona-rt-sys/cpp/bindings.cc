// TODO: Include what you use.
// std
#include <array>
#include <ostream>
#include <stddef.h>
#include <stdint.h>
#include <string_view>
// verona
#include <cpp/lambdabehaviour.h>
#include <object/object.h>
#include <sched/cown.h>
#include <sched/schedulerthread.h>

using verona::rt::Behaviour;
using verona::rt::BehaviourCore;
using verona::rt::Cown;
using verona::rt::Descriptor;
using verona::rt::Object;
using verona::rt::Scheduler;
using verona::rt::Work;

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

typedef void (*WhenNFunc)(size_t, Cown**, void*);

template<size_t N_COWNS>
struct CownThunk
{
  std::array<Cown*, N_COWNS> cowns_;
  WhenNFunc thunk_;
  void* data_;

  void operator()()
  {
    // TODO: remove size param.
    thunk_(N_COWNS, cowns_.data(), data_);
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
  assert(len == N_COWNS);
  auto cownarr = gather_cown<N_COWNS>(len, ptr);
  auto lambda = CownThunk<N_COWNS>{cownarr, func, data};
  verona::rt::schedule_lambda(N_COWNS, ptr, std::move(lambda));
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

#define BUILD_SCHEDULE(n) \
  void boxcars_sched_##n(size_t len, Cown** ptr, WhenNFunc func, void* data) \
  { \
    assert(len == n); \
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
  BehaviourCore* boxcars_behaviourcore_from_work(Work* work)
  {
    return BehaviourCore::from_work(work);
  }
  void* boxcars_behaviourcore_get_body(BehaviourCore* be)
  {
    return be->get_body();
  }
  void boxcars_behaviourcore_release_all(BehaviourCore* be)
  {
    be->release_all();
  }
  void boxcars_work_dealloc(Work* work)
  {
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

  void boxcars_snmalloc_message(const char* ptr, size_t len)
  {
    std::string_view s(ptr, len);
#ifdef SNMALLOC_TRACING
    snmalloc::message<1024>("{}", s);
#endif
  }
}
