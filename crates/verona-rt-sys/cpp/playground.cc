#include <cpp/when.h>
#include <debug/harness.h>
#include <iostream>

#define log(msg) do_log((msg), __FILE__, __LINE__)

void do_log(const char* msg, const char* file, size_t line)
{
  std::cerr << file << ":" << line << " " << msg << std::endl;
}

using verona::cpp::make_cown;
using verona::cpp::when;

void real_main()
{
  auto a = make_cown<int>(101);

  log("begin b0");

  when(a) << [](auto a2) {
    log("begin b1");
    when(a2.cown()) << [](auto a3) { log("done b3"); };
    log("end b1");
  };

  log("end b0");
}

int main(int argc, const char* const* argv)
{
  auto& x = verona::rt::Scheduler::get();
  x.init(1);
  auto a = make_cown<int>(101);
  x.run();
}