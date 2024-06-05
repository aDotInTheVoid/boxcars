// TODO: Include what you use.
// std
#include <array>
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

// Sane Rust platform assumptions.
static_assert(sizeof(void*) == sizeof(size_t));
static_assert(sizeof(void*) == sizeof(ptrdiff_t));

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

  void bbench_busy_loop(size_t n)
  {
    busy_loop(n);
  }

  // TODO: Don't put these in main binary
  void bbench_create_n_cowns(size_t n)
  {
    using verona::cpp::make_cown;

    std::vector<verona::cpp::cown_ptr<size_t>> v;
    v.reserve(n);

    for (int i = 0; i < n; i++)
    {
      v.push_back(make_cown<size_t>(i));
    }
  }

  void bbench_schedule_n_lambdas_onto_cown(size_t n)
  {
    Scheduler::get().init(1);

    auto c = verona::cpp::make_cown<int32_t>(0);

    for (int i = 0; i < n; i++)
    {
      verona::cpp::when(c) << [](auto c) { c++; };
    }

    Scheduler::get().run();
  }

  void bbeench_busyloop_inside_when(size_t n)
  {
    Scheduler::get().init(1);

    auto c = verona::cpp::make_cown<size_t>(n);
    verona::cpp::when(c) << [](auto c) { busy_loop(*c); };

    Scheduler::get().run();
  }
}
