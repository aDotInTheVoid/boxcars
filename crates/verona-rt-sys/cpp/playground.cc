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

class Foo
{
  int number_;
  const char* str_;

public:
  Foo(int number, const char* str) : number_(number), str_(str) {}
};

void real_main()
{
  // These are the wrong way round
  auto c_foo = make_cown<Foo>("hello", 101);
}

int main(int argc, const char* const* argv)
{
  auto& x = verona::rt::Scheduler::get();
  x.init(1);
  auto a = make_cown<int>(101);
  x.run();
}