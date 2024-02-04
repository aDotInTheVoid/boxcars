---
title: "BoC in Rust: Interim Report"
author: Alona Enraght-Moony
date: 2024-01-23
bibliography: cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
papersize: a4
geometry: margin=2cm
---

## Background

### Rust

Rust [@rust_book] is a programming languages originally developed by Mozilla Research,
and currently maintained by a large cross-org team.

Rust's most important feature (for our purposes) is its system of **Ownership & Borrowing**.

#### Ownership & Borrowing: A very fast introduction.

(A full tutorial on ownership and borrowing is beyond the scope of this report. See [@rust_book] for details.)
The core idea of **ownership** is that each value has a unique owner, and that value is dropped when the
owner goes out of scope. To quote The Book [@rust_book]:

> - Each value in Rust has an owner.
> - There can only be one owner at a time.
> - When the owner goes out of scope, the value will be dropped.

However with only these rules, programming in Rust would be extremely
uneconomic. If a function took a string as a parameter, it would have "take
ownership" of that value, so the calling function could no longer use it.
[^return_values]

[^return_values]: This could be circumvented by having a function return back it's args to the caller, but this would
be extremely cumbersome.

Therefore, Rust introduces the additional notion of **borrowing**. A value can be borrowed in
one of two ways: by a shared reference (spelt `&T`), or an exclusive reference (`&mut T`). These are
also sometimes referred to as a immutable or mutable reference respectively [@dtolnay_ref].

A value may have many shared references to it at a given time, but if it has any
exclusive reference to it that reference must be the only one. With a shared
reference, you can only read from the value. An exclusive reference is required
to mutate it. More succinctly, Rust references are "Aliasable XOR mutable" [@boats_smaller].

Borrowed values have a **lifetime** for which they are borrowed. This is needed to ensure
that all exclusive references don't overlap with shared ones.

```rust
let x: i32 = 0;

let shared_1: &i32 = &x; // Lifetime 1 starts
dbg!(shared_1);
// Lifetime 1 stops.

let exclusive_2: &mut i32 = &mut x; // Lifetime 2 starts
// Would be compiler error to borrow x here.
*exclusive_2 += 10;
// Lifetime 2 ends

// But now that lifetime 2 is over, we can borrow again

let shared_3: &i32 = &x; // Lifetime 3 starts
let shared_4: &i32 = &x; // Lifetime 4 starts

dbg!(shared_4);
// Lifetime 4 ends
dbg!(shared_3);
// Lifetime 3 ends
```

This is also used to ensure that references don't outlive the objects they borrow.

```rust
let mut x: &i32 = 0;
{
    let short_lived: i32 = 0;
    x = &short_lived;
} // short_lived goes out of scope here.
dbg!(x);
```

will error with

```
error[E0597]: `short_lived` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let short_lived: i32 = 0;
  |             ----------- binding `short_lived` declared here
5 |         x = &short_lived;
  |             ^^^^^^^^^^^^ borrowed value does not live long enough
6 |     }
  |     - `short_lived` dropped here while still borrowed
7 |     dbg!(x);
  |          - borrow later used here
```

instead of doing UB like it would in C++.

#### Consequence of this: No iterator invalidation

(Note, these examples adapted from "Two Beautiful Rust Programs" [@matklad_beautiful], because otherwise
I'd be recreating them from memory)

https://matklad.github.io/2020/07/15/two-beautiful-programs.html

```Rust
fn main() {
  let mut xs = vec![1, 2, 3];
  let x: &i32 = &xs[0];
  xs.push(92);
  println!("{}", *x);
}
```

```
error[E0502]: cannot borrow `xs` as mutable because it is also borrowed as immutable
 --> <source>:4:3
  |
3 |   let x: &i32 = &xs[0];
  |                  -- immutable borrow occurs here
4 |   xs.push(92);
  |   ^^^^^^^^^^^ mutable borrow occurs here
5 |   println!("{}", *x);
  |                  -- immutable borrow later used here

error: aborting due to previous error
```

whereas in C++

```cpp
#include <vector>
#include <iostream>

int main() {
    std::vector<int> xs {1, 2, 3};
    int* x = &xs[0];
    xs.push_back(4);
    std::cout << *x << '\n'; 
}
```

does undefined behavior.

This is because when we call `.push_back()`, the vector needs to reallocate, as it initially
only had enough capacity for 3 elements. After moving the values to the new allocation, it frees
the old one. This means `x` is a dangling pointer, and dereferencing it is a heap use-after-free.

Indeeed, we can see this by using Asan, which can show up a stack trace of when this happend.

```
=================================================================
==26780==ERROR: AddressSanitizer: heap-use-after-free on address 0x602000000010 at pc 0x55dc3792032f bp 0x7ffcdf94ec50 sp 0x7ffcdf94ec48
READ of size 4 at 0x602000000010 thread T0
    #0 0x55dc3792032e in main /home/alona/tmp/./bad.cpp:8:18
   
0x602000000010 is located 0 bytes inside of 12-byte region [0x602000000010,0x60200000001c)
freed by thread T0 here:
    #0 0x55dc3791e131 in operator delete(void*) (/home/alona/tmp/a.out+0xf6131) (BuildId: ba94dc30bfb525b0467634abdb5f45d077025f55)
    #6 0x55dc379207fc in std::vector<int, std::allocator<int>>::push_back(int&&) /usr/bin/../lib/gcc/x86_64-linux-gnu/13/../../../../include/c++/13/bits/stl_vector.h:1296:9
    #7 0x55dc379202de in main /home/alona/tmp/./bad.cpp:7:8
  
previously allocated by thread T0 here:
    #0 0x55dc3791d8b1 in operator new(unsigned long) (/home/alona/tmp/a.out+0xf58b1) (BuildId: ba94dc30bfb525b0467634abdb5f45d077025f55)
    #5 0x55dc379206bd in std::vector<int, std::allocator<int>>::vector(std::initializer_list<int>, std::allocator<int> const&) /usr/bin/../lib/gcc/x86_64-linux-gnu/13/../../../../include/c++/13/bits/stl_vector.h:679:2
    #6 0x55dc37920234 in main /home/alona/tmp/./bad.cpp:5:22
```

We allocated some memory to create the vector, took a reference to it,
 but then freed it when we did the `push_back()`, and
then used the it afterwards.



Currently on my machine it prints a seemingly random value.
However modifying to so that the vector is larger (`std::vector<int> xs (1000000, -1);`),
makes the program segfault instead. The exact details of what happens here are at the whims
of the compiler and the standard library

#### Consequence of this: Fearless concurrency

```Rust
use std::thread::scope;
use std::sync::{Mutex, MutexGuard};

fn main() {
  let mut counter = Mutex::new(0);

  scope(|s| {
    for _ in 0..10 {
      s.spawn(|| {
        let mut guard: MutexGuard<i32> = counter.lock().unwrap();
        *guard += 1;
      });
    }
  });

  let total: &mut i32 = counter.get_mut().unwrap();
  *total += 1;
  println!("total = {total}");
}
```

1. Each thread only accesses the counter through the mutex.
2. Each thread can read from `main`'s stack.
3. Final increment doesn't need to lock mutex, no other threads could have it.

If any of these properties had changed, we'd get a compiler error.

#### Ownership & Borrowing can't save you from everything

However, one area where Rust's type system doesn't do anything to help you is avoiding deadlocks.
One can trivially perform one by acquiring two mutex's in different orders:

```Rust
use std::sync::Mutex;
use std::thread::{scope, sleep_ms};

fn main() {
    let m1 = Mutex::new(());
    let m2 = Mutex::new(());

    scope(|s| {
        s.spawn(|| {
            let g1 = m1.lock();
            sleep_ms(100);
            println!("t1: got m1, trying to get m2");
            let g2 = m2.lock();
            println!("t1: got both");
        });

        s.spawn(|| {
            let g2 = m2.lock();
            sleep_ms(100);
            println!("t2: got m2, trying to get m1");
            let g1 = m1.lock();
            println!("t2: got both");
        });
    });
}
```

While this example is obviously contrived, it demonstrates an important point.
Rust deadlocks have come up in practice, and are usually much more subtle than this
[@snoyman_deadlock; @fasterthanlime_deadlock].

### Behaviour-Oriented Concurrency

Behaviour-Oriented Concurrency (BoC) is a novel concurrency paradigm
[@when_concurrency_matters]. It has 2 key components:

- **Cowns**: A cown (short for concurrent owner) is a piece of data.

    A cown can be in one of two states: available or acquired. An available
    cown is eligible to be acquired by a behaviour, but it's associated data
    cannot be accessed. An acquired cown can have it's data accessed, but only
    while it's aquired.

    The only way to acquire a cown (and thus access it's) is to run a behaviour on it.

- **Behaviours**: A behaviour is a unit of execution that acts upon a set of cowns.

    When you create a behaviour, you give the set of cowns it acts on, as well as
    the code to run. When all the required cowns can be acquired, the code is executed, and then
    the cowns are made available again.

    Cowns can only be acquired by one behaviour at once. This can be thaugh of as being a bit
    like each Cown having a mutex, which is locked before the behaviour starts and unlocked after
    it ends. However, every cown in a behaviour is acquired "at once"

    Note that this means that creating a behaviour returns immediately, and the code inside
    will be executed at some indetermined future point, when unique access to all cowns can
    be guaranteed.


```scala
var myCown: Cown[int] = cown.create(10);

when(myCown) {
    myCown += 10;    
}

myCown += 10; // invalid.
```

BoC is both *data-race free* and *deadlock free*. No data races can occur, as
cowns can only be modified when acquired by behaviours,

### Verona Runtime

The verona runtime (sometimes also known as `verona-rt`) is a C++ library that
implements behaviour oriented concurrency. It is intended to be a part of the
currently in development Verona language, but it can also be used as a
freestanding C++ library today.

It uses the C++ type system to enforse the destinction between availible and 
aquired cowns. The `cown_ptr<T>` type references a cown containing `T`, but doens't
allow accessing the data. Instead, when a behaviour executed, it's given an `acquired_cown<T>`, 
which can access the underlying data.

Internally, these have the same representation, but. An `acquired_cown<T>` is
only handed out when the behavior runs. In this way, the compiler can enforce that
cowns can only have their data accessed when acquired. Eg:

```cpp
cown_ptr<int> my_cown = make_cown<int>(10);
when(my_cown) << [](acquired_cown<int> my_cown) {
    my_cown += 10; // this works fine, via operator overloading.
};
my_cown += 10; // compiler error!
```

The `when` function takes a `cown_ptr`, and a lambda that takes an `acquired_cown`.
When the cown can be acquired, it creates an `acquired_cown` (which cannot be done outside the verona-rt library),
and passes that to the lambda. 

However, a determined user can circumvent this type safety. As one example:

```cpp
cown_ptr<int> my_cown = make_cown<int>(10);
int* escape_data;
when(my_cown) << [&escape_data](acquired_cown<int> my_cown) {
    escape_data = &*my_cown;
};
sleep(3); // Wait for behaviour to run
*escape_data += 10; // Modifies cown without acquiring!!!
```

This exposes the underlying data via `escape_data`. A user could then
modify the cowns data without acquiring it, undermining BoC's goal
of no-data-races. This is because C++ has no mechanism to limit
how long a pointer can be used, and the `acquired_cown` class
must give out pointers to allow access to the underlying data.

Rust can solve this with the power of lifetimes.

<!--
### Other concurrency paradimes

- Shared Memory
- Message Passing
- Fork/Join
- Actors
- Structured Concurrency
-->

## The work done so far

So far, I've completed Rust bindings to the core functionality of the verona-rt
library. Let's start with an example.

```Rust
let string = CownPtr::new(String::new());
let vec = CownPtr::new(Vec::new());

when(&string, |mut s| {
    assert_eq!(&*s, "");
    s.push_str("foo");
});
when(&vec, |mut v| {
    assert_eq!(&*v, &[]);
    v.push(101);
});
when2(&string, &vec, |mut s, mut v| {
    assert_eq!(&*s, "foo");
    assert_eq!(&*v, &[101]);
    s.push_str("bar");
    v.push(666);
});
when(&string, |s| assert_eq!(&*s, "foobar"));
when(&vec, |v| assert_eq!(&*v, &[101, 666]));
```

This is very similar to the equivalent C++ code.

```cpp
auto string = make_cown<std::string>();
auto vec = make_cown<std::vector<int>>();

when(string) << [](auto s) {
  assert(*s == "");
  s->append("foo");
};
when(vec) << [](auto v) {
  assert(v->size() == 0);
  v->push_back(101);
};
when(string, vec) << [](auto s, auto v) {
  assert(*s == "foo");
  assert(*v == std::vector{101});
  s->append("bar");
  v->push_back(666);
};
when(string) << [](auto s) { assert(*s == "foobar"); };
when(vec) << [](auto v) { assert((*v == std::vector{101, 666})); };
```

This works today.

## Design

### Reusing `verona-rt`

I've reuse the exiting `verona-rt` library.

While it would be possible to implement a new runtime for BoC, theirs an existing,
well-tested implementation in C++. Writing bindings to it allows reusing the existing
design, testing and optimization work done in it. It also allows the possibility of
sharing Cowns between Rust and C++.

Instead, we implement a Rust library that makes FFI calls to the C++ verona-rt library.
This is challenging, as Rust only supports FFI calls to C functions, not C++. Therefor
I've written a adaptor layer that exposed the verona-rt functionality via `extern "C"`
functions.

This means the Rust layer is relatively thin, as it doesn't need to implement behavior
scheduling, but merely needs to expose a ergonomic API, and then translate it into
calls into verona-rt.

### Generics across the FFI boundry.

When using the verona-rt library in C++, the entire thing is a header only library.
When you use a `cown_ptr<UserDefinedType>`, the compiler has full knowledge of both
`cown_ptr` and `UserDefinedType`, so it can do template expansion, and generate all the
necessary code.

However, if we want to do this for a Rust type, the Rust compiler doesn't know how
`cown_ptr` is defined, so it can't do template expansion. Therefore a bit of creativity
is needed to allow the library to support generics like the C++ library can.

Were the entire runtime in Rust, we could just use Rust generics without
restriction, and the compiller would monomorphize whatever functions we needed.

#### Basic Approach: Don't allow generics.

My initial version solved this by not allowing generic cowns. Instead we just
allowed `cown_ptr<int32_t>` (and friends). This makes everything much simpler.

```cpp
extern "C" {

void cown_int_new(int32_t value, cown_ptr<int32_t>* out) {
    *out = make_cown<int32_t>(value);
}

typedef void(use_int(acquired_cown<int32_t>* cown, void* data));

void cown_int_when1(const cown_ptr<int32_t>* cown, use_int func, void* data) {
    when(cown) << [=](auto cown) { func(cown, data); };
}

}
```

These functions aren't templated, and are `extern "C"`, so we can call them from
Rust. While this works if you only want `i32` cowns, you'd need to duplicate
this work for every other type you wanted. While this would be feasible (if a
bit impractical) to do for the rest of the builtin types, it wouldn't work for
user defined types, as their definition (and in particular their size), wouldn't
be known when the verona-rt library is compiled.

#### Approach that works: pass the size at runtime

The core problem is that the C++ compiler needs to know the size of each object,
so that it can be layed out in memory, and it knows what offsets to place each
of the fields at, and how much stack space to reserve. However, the underlying
Cown data (as well as the associated rust data) is placed on the heap, and C++
only needs to know about the Cown metadata, not the Rust user data.

This means we can pass in the size of the allocation, and C++ will only use the start
of it to store the Cown metadata, and the rest is free for rust to use.

```
          | +--------+
Allocation| | Cown   |<------cown_ptr
size only | | Data   |
known at  | +--------+
runtime   | | Rust   |
          | | Struct |
          | |        |
          v +--------+
```

We can dynamical allocate enough memory to hold the Rust struct. Then C++ only
uses a fixed size of it, to store the cown data (reference count, scheduling
information, etc). The tailing portion is exclusively used by Rust to store whatever
information is needed.

On the Rust side this looks like:

```Rust
#[repr(C)]
struct ActualCown {
    _marker: MaybeUninit<[*const (); 4]>,
}

#[repr(C)]
struct CownData<T> {
    // Must be first, so we can convert pointers between the two.
    cown: ActualCown,
    data: T,
}

struct CownPtr<T> {
    cown: *const CownData<T>
}
```

When creating a new `CownPtr<T>`, we pass in the size of `T` across the FFI boundary,
where the C++ side allocates enough space to store the `T` (as well as the cown metadata).
C++ then fills in the cown metadata, and returns a pointer to the allocation. The Rust side
can then do pointer arithmetic to get a pointer to the uninitialized memory where the `T` should
go, and write it to their.

This means that the `CownPtr<T>::new` doesn't need to call a templated C++ function, so it can be a simple
`extern "C"` function, callable from Rust.

### Updating the reference count.

Each cown has a reference count, to keep it alive until their are no outstanding
references. In C++, this is maintained by having `cown_ptr<T>` overwrite it's copy,
assignment, move constructor, move-assignment constructor and destructor to keep the
reference count up to date. We'd need to do the same thing in Rust.

I chose early on to reuse this C++ reference count, instead of implementing my own one
in Rust (or using the `std::sync::Arc` type). This is for the same reasons that I chose
to reuse the C++ schedular. However it presents some challenges, as Rust doesn't have the
same language level concepts that are used in the C++ implementation, so some ingenuity is
required.

The most basic case in a simple assignment. In C++, you can 


```cpp
template <class T>
class cown_ptr {
    ActualCown<T>* allocated_cown{nullptr};

    void clear() {
      // Sets the cown_ptr to nullptr, and decrements the reference count
      // if it was not already nullptr.

      if (allocated_cown != nullptr) {
        release(allocated_cown);
        allocated_cown = nullptr;
      } 
    }

    // 1. copy constructor
    cown_ptr(const cown_ptr& other)
    {
      allocated_cown = other.allocated_cown;
      if (allocated_cown != nullptr)
        acquire(allocated_cown);
    }

    // 2. copy assignment
    cown_ptr& operator=(const cown_ptr& other) {
      clear();
      allocated_cown = other.allocated_cown;
      if (allocated_cown != nullptr)
        acquire(allocated_cown);
      return *this;
    }

    // 3. move constructor
    cown_ptr(cown_ptr&& other)
    {
      allocated_cown = other.allocated_cown;
      other.allocated_cown = nullptr;
    }

    // 4. move assignment
    cown_ptr& operator=(cown_ptr&& other)
    {
      clear();
      allocated_cown = other.allocated_cown;
      other.allocated_cown = nullptr;
      return *this;
    }

    // 5. destructor
    ~cown_ptr() { clear() }
};
```

1. **copy constructor**
    
    In C++, this copies the pointer, and then bumps the reference count.

    Rust has no direct equivalent, but the `Clone::clone` method is pritty close.
    It's fairly simple to `impl<T> Clone for CownPtr<T>`, by calling into C++ code
    that bumps up the reference count, and then returning the same pointer.

2. **copy assignment**

    C++ lets you overload assignment. This is super useful for reference counted pointers.

    ```cpp
    cown_ptr<int> a = make_cown(1);
    cown_ptr<int> b = make_cown(1);

    a = b; // <- needs to update reference counts.
    ```

    On the assignment, instead of just moving memory, happen:

    1. `a`'s reference count is decremented
    2. `b`'s pointer is also stored into `a`.
    3. This new reference count is incremented.

    On the other hand, in Rust, your unable to overload assignment. It's always
    a bitwise copy (and possibly droping the old value). Theirs no way to run no
    user proved code, so are we stumped here? No! A key reason C++ needs to be
    complex here this that after the assignment runs, both `a` and `b` need to
    be valid for use. Whereas in Rust, this'd transfer ownership, and it'd be a
    compiler error. C++ also will still run `b`'s destructor after this, whereas
    Rust won't. This essentially means that all Rust assignments are move-assignment,
    so we don't need to worry about this case [^copy]. The clostest equivalent would
    be

    ```Rust
    let b: CownPtr<i32> = a.clone();
    ```

    but this will

    1. Run `b`'s destructor.
    2. Calls `a`'s `.clone()` method.
    3. Assign that result into `b`.

    Rust doesn't let you have one function that does all of this, it must be composed out
    of your `Drop` and `Clone` impls.

[^copy]: Except for types which implement the `Copy` trait, but that doesn't apply here.

3. **move constructor**
    
    The C++ move constructor doesn't touch the reference count, but does set the
    moved-from pointer to `nullptr`. This is needed as C++ still runs the destructor
    for moved-from objects.

    However, Rust doesn't do this, as moved-from objects are no longer valid to access.
    Therefor, we can use a bitwise copy here, and things will be fine. This is fortunate
    for us, as that's the only semantics Rust will let us give

    ```Rust
    let a: CownPtr<i32> = b;
    ```
    
4. **move assignment**

    The move assignment constructor first `clear()`s the value being assigned to.
    This is needed to reduce it's reference count. The rest is equivalent to the 
    move constructor.

    This once again allows us to have something that works. Rust will compile the assignment in:

    ```Rust
    let mut a = CownPtr::new(1);
    let mut b = CownPtr::new(2);
    a = b; // <- drops a, then does bitwise copy.
    ```

    to first dropping the old value of `a`, then bitwise copying `b` into `a`, and then marking `b`
    as moved-from so it's drop code won't get run.

5. **destructor**

    The destructor decrements the reference count. This can be achived by implementing the
    `std::ops::Drop` trait, which will run out code when a value goes out of scope (or otherwise
    needs to be destructed).


Therefor, we can have a Rust `CownPtr<T>` struct that successfully keeps the reference
count up to date, even tough it can't overload all the constructors and operators that the
C++ equivalent can.

The main downside to this is that we rely on implementation details of the C++
`cown_ptr` type. If the `verona-rt` changed how the constructos were implemented,
users of the C++ code would transparently use these new impls. However, this change
may break the Rust code, with no easy path for migration.

Another downside is that both `Clone` and `Drop` calls require doing FFI into
C++. This may cause performance overhead, as the compiler won't be able to do
inlining here. However this is purely speculative, as I've not benchmarked the
current implementation. In the case that this causes a notisable slowdown, 
cross-language LTO [@clang_xlto] may solve this, but again this can't be known
without benchmarking.


## Evaluation Plan

- Reuse benchmarks from BoC paper, to show low overhead.
- Subjectively, they also show something about expressiveness

## Ethical Issues

This project presents very few ethical issues. The only thing to be concerted
about is clear attribution of work, as it builds extensively on pre-existing
research into Behavior Oriented Concurrency, and it's implementation in the
Verona Runtime. It must be clear what of the work was my own, as what wasn't.

This is made more complex by the fact that I have a direct dependency on the
verona-rt library, and have made modification to it to support my specific use
case. I'll also (hopefully) end up conversing with MSR people about how to
integrate these changes upstream, and other possible directions. I need to be
careful to properly acknowledge this potential collaboration (should it occur).

## Bibliography

<!-- pandoc magically fills this in -->


<!-- 
- Who Owns RefCounting
    - Cost: Hard to inline refcounting
        - Potential fix: X-lang LTO
            - This complicates the build
            - Benchmark this
            - Benchmark Rust vs C++
            - Savina benchmarks.
    - Cost: Store a dtor pointer in every object.
        - Potenial Fix: Pass dtor pointer on Drop call
        - Still has Nop call for Cowns with nop dtors
        - Althoug Xlang LTO could fix this (needs benchmarking).
    - Design Cost: We assume a load about the internals.
        - These arn't garenteed by the API.
        - Sometimes need a +9 object size.
        - Maintences concerns after I stop.
- Be no_std / have no_allocs
    - Lets you use their embeded runtime.
    - https://github.com/microsoft/monza

 -->