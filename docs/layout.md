# Memory Layout.

It's usefull to understand how verona-rt lays thing out in memory.

## C++ Types

```
clang++ ./bindings.cc -I ../../../verona-rt/src/rt/ -I ./build/_deps/snmalloc-src/src/ -mcx16 -latomic -Xclang -fdump-record-layouts > a-bindings.cc.001l.class
```

### `ActualCown`

```
*** Dumping AST Record Layout
 0 | class verona::cpp::ActualCown<class verona::cpp::DtorThunk>
 0 |   class verona::rt::VCown<class verona::cpp::ActualCown<class verona::cpp::DtorThunk> > (base)
 0 |     class verona::rt::VBase<class verona::cpp::ActualCown<class verona::cpp::DtorThunk>, class verona::rt::Cown> (base)
 0 |       class verona::rt::Cown (base)
 0 |         class verona::rt::Shared (base)
 0 |           class verona::rt::Object (base) (empty)
 0 |           struct std::atomic<unsigned long> weak_count
 0 |             struct std::__atomic_base<unsigned long> (base)
 0 |               __int_type _M_i
 8 |         struct std::atomic<struct verona::rt::Slot *> last_slot
 8 |           struct std::__atomic_base<struct verona::rt::Slot *> _M_b
 8 |             __pointer_type _M_p
16 |         struct verona::rt::ReadRefCount read_ref_count
16 |           struct std::atomic<unsigned long> count
16 |             struct std::__atomic_base<unsigned long> (base)
16 |               __int_type _M_i
24 |   class verona::cpp::DtorThunk value
24 |     dtor dtor_
   | [sizeof=32, dsize=32, align=8,
   |  nvsize=32, nvalign=8]
```

## `cown_ptr`

```
*** Dumping AST Record Layout
0 | class verona::cpp::cown_ptr<class verona::cpp::DtorThunk>
0 |   class verona::cpp::cown_ptr_base (base) (empty)
0 |   ActualCown<DtorThunk> * allocated_cown
  | [sizeof=8, dsize=8, align=8,
  |  nvsize=8, nvalign=8]
```

This a pointer to an `ActualCown` + some semantics. Semanticly, it must update the reference count.

### `aquired_cown`

```
*** Dumping AST Record Layout
         0 | class verona::cpp::acquired_cown<class verona::cpp::DtorThunk>
         0 |   ActualCown<std::remove_const_t<DtorThunk> > & origin_cown
           | [sizeof=8, dsize=8, align=8,
           |  nvsize=8, nvalign=8]
```

This a pointer to an `ActualCown` + some semantics.

## Assumptions we make for the pointers.

- It's fine to move `aquired_cown`, as long as you don't copy/clone it.
  - The C++ impl `delete`s all assignment/move/copy operators.
  - But we can't enforce this in Rust.
  - But we can enforse no use-after-move, which C++ can't.
  - And as long as no-one takes the address (which no-one does), it's all OK.
- It's fine to move a `cown_ptr` without calling code.
  - In C++, the copy-assignment/constructor increments the ref count:
    - Rust doens't have this operator, but we use `Clone` to achieve a similar effect.
  - C++ move assignent/constructor doesn't touch the ref-count, but sets the old pointer to null.
    - Rust doesn't have this operator, but we use `Drop` to decrement the ref-count.
    - We don't need to change the move-constructor, because rust doesn't call dtors on moved-from values (but C++ does).

## `ActualCown` shenanigans.

`ActualCown` is the place in C++ that holds the data protected by the `Cown` and also the Cown's data itself.

It can be thaught of as.

```c++
template <typename T>
class ActualCown {
  Cown cown_; // Holds ref-count, scheduler into.
  T value_; // Holds the data.
};
```

The problem is we can't instantiate this template in Rust (across an FFI
boundry), so we need to restort to some trickery.

First we delcare a C++ cown type that runs a user provided destructor.

```C++
typedef void (*dtor)(void *);

class DtorThunk {
  dtor dtor_;
  ~DtorThunk() { dtor_(this); }
};

using ActualCown = verona::rt::ActualCown<DtorThunk>;
```

Then we define a rust type that represents this type, but is completely opaque to rust.

```rust
#[repr(C)]
struct ActualCown([usize; 4]); 
```

Now we can define a wrapping type that is generic.

```rust
#[repr(C)]
pub(crate) struct CownDataToxic<T> {
    cown: ActualCown,
    data: T,
}
```

Because `ActualCown` is the first field (and it's `repr(C)`) it's safe to cast
from a `*mut CownDataToxic<T>` to a `*mut ActualCown` (and back, assuming you
know it's embeded).

We need to hook into the verona runtime so we can create an `ActualCown` but
with enough space for the `CownDataToxic<T>`. When we do this, we also need to
add 16 bytes at the end of the allocation for the object header (which appears
to be stored at the end of the allocation). See the [this][operator_new] and
[this][vsizeof] for where verona does the allocations.

[operator_new]: https://github.com/microsoft/verona-rt/blob/0919daa4a6053773d3b74bb3c82702c202175630/src/rt/cpp/vobject.h#L181-L185
[vsizeof]: https://github.com/microsoft/verona-rt/blob/0919daa4a6053773d3b74bb3c82702c202175630/src/rt/object/object.h#L868C1-L870
