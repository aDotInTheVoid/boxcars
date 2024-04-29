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

typedef void (*WhenNFunc)(size_t, Cown**, void*);
typedef void (*When1Func)(Cown*, void*);
typedef void (*When2Func)(Cown*, Cown*, void*);

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
    std::cerr << "Checking for leaks" << std::endl;
    snmalloc::debug_check_empty<snmalloc::Alloc::Config>(&is_ok);
    std::cerr << "leak check done. is_ok=" << is_ok << std::endl;
    return !is_ok;
  }

  bool schedular_has_leaks()
  {
    verona::rt::LocalEpochPool::sort();

    bool has_leaks = get_has_leaks();

    if (has_leaks)
    {
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
  void boxcars_schedule_1(Cown* cown, When1Func func, void* data)
  {
    verona::rt::schedule_lambda(cown, [=]() { func(cown, data); });
  }
  void boxcars_schedule_2(Cown* c1, Cown* c2, When2Func func, void* data)
  {
    Cown* cowns[2] = {c1, c2};
    verona::rt::schedule_lambda(2, cowns, [=]() { func(c1, c2, data); });
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
}
