
### Consequence of this: No iterator invalidation

(Note, these examples adapted from "Two Beautiful Rust Programs" [@matklad_beautiful], because otherwise
I'd be recreating them from memory)

https://matklad.github.io/2020/07/15/two-beautiful-programs.html

```rust
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

### Consequence of this: Fearless concurrency

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
