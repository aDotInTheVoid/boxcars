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

using verona::rt::Cown;
using verona::rt::Descriptor;
using verona::rt::Object;
using verona::rt::Scheduler;
using verona::rt::VCown;

// TODO: Remove these
using cown_ptr = verona::cpp::cown_ptr<int>;
using acquired_cown = verona::cpp::acquired_cown<int>;
using ActualCown = verona::cpp::ActualCown<int>;

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
    size_t size = desc->size + 1000; // TODO: Don't do this :(
    void* base = snmalloc::ThreadAlloc::get().alloc(size);

    Logging::cout() << "Allocated " << size << " bytes cown at " << base
                    << Logging::endl;

    Object* obj = Object::register_object(base, desc);

    Cown* cown = new (obj) Cown();

    Logging::cout() << "Registeded cown at address " << cown << Logging::endl;

    return cown;
  }

  // void
  // boxcar_when1(cown_ptr* cown, void (*func)(acquired_cown*, void*), void*
  // data)
  // {
  //   when(*cown) << [=](acquired_cown acq) { func(&acq, data); };
  // }
  // void boxcar_when2(
  //   cown_ptr* c1,
  //   cown_ptr* c2,
  //   void (*func)(acquired_cown*, acquired_cown*, void*),
  //   void* data)
  // {
  //   when(*c1, *c2) << [=](auto a1, auto a2) { func(&a1, &a2, data); };
  // }

  void boxcars_test_descriptor_info(size_t* size, size_t* align)
  {
    *size = sizeof(Descriptor);
    *align = alignof(Descriptor);
  }
}
