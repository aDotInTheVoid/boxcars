#include <cpp/when.h>
#include <debug/harness.h>
#include <iostream>
#include <vector>

#define log(msg) do_log((msg), __FILE__, __LINE__)

void do_log(const char* msg, const char* file, size_t line)
{
  std::cerr << file << ":" << line << " " << msg << std::endl;
}

using verona::cpp::make_cown;
using verona::cpp::when;

int main(int argc, const char* const* argv)
{
  auto& x = verona::rt::Scheduler::get();
  x.init(2);
  log("hello");

  std::vector<int> my_vec{1, 2, 3};
  auto& vec_ref = my_vec;
  auto cown = make_cown<std::vector<int>>(std::move(my_vec));
  vec_ref[0] = 4;

  when(cown) << [](auto c) {
    log("exec");
    for (auto i : *c)
    {
      std::cout << i << " ";
    }
  };

  log("done");
  x.run();
}