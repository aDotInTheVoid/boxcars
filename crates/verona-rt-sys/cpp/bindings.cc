#include <cpp/cown.h>
#include <cpp/when.h>
#include <iostream>

using verona::cpp::acquired_cown;
using verona::cpp::cown_ptr;
using verona::cpp::make_cown;
using verona::cpp::when;
using verona::rt::VCown;

struct Account : public verona::rt::VCown<Account>
{
  // Need explicit ctor for make_cown template magic.
  Account(int balance, bool frozen) : balance_(balance), frozen_(frozen) {}

  int balance_;
  bool frozen_;
};

typedef void(use_account(acquired_cown<Account>&));
typedef void(use_accout2(acquired_cown<Account>&, acquired_cown<Account>&));

extern "C"
{
  cown_ptr<Account> make_account(int balance, bool frozen)
  {
    static_assert(sizeof(cown_ptr<Account>) == sizeof(Account*));
    return make_cown<Account>(balance, frozen);
  }

  void when_account(cown_ptr<Account> account, use_account func)
  {
    when(account) << [func](auto acc) { func(acc); };
  }

  void
  when_account2(cown_ptr<Account> a1, cown_ptr<Account> a2, use_accout2 func)
  {
    when(a1, a2) << [=](auto a1, auto a2) { func(a1, a2); };
  }

  cown_ptr<Account> cown(acquired_cown<Account> acc)
  {
    return acc.cown();
  }
}
