---
title: "Boxcars: BoC in Rust"
# author: "Alona Enraght-Moony <ne321@imperial.ac.uk>"
date: 2024-02-06
aspectratio: 169
---

## What do we want to do here?

- BoC concurrency in Rust.
- With the existing verona-rt implementation.

##

```rust
let string = CownPtr::new(String::new());
let vec = CownPtr::new(Vec::<i32>::new());
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

##

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
when(vec) << [](auto v) { assert((*v == std::vector{101, 666})); }
```


##

```cpp
auto string = make_cown<std::string>();
auto vec = make_cown<std::vector<int>>();
when( string) << [](auto s) {
    assert    ( *s == "");
    s->append ("foo");
};
when( vec) << [](auto v) {
    assert   (v->size() == 0);
    v->push_back(101);
};
when ( string,  vec) << [](auto s, auto v) {
    assert    ( *s == "foo");
    assert    ( *v == std::vector{101});
    s->append ("bar");
    v->push_back(666);
};
when( string) << [](auto s) { assert    ( *s == "foobar"); };
when( vec) << [](auto v) { assert    ((*v == std::vector{101, 666})); };
```

## 

```rust
let  string = CownPtr::new(String::new());
let  vec = CownPtr::new(Vec::<i32>::new());
when(&string,       |mut s| {
    assert_eq!(&*s,   "");
    s.push_str("foo");
});
when(&vec,       |mut v| {
    assert_eq!(&*v, &[]);
    v.push      (101);
});
when2(&string, &vec,       |mut s,  mut v| {
    assert_eq!(&*s,   "foo");
    assert_eq!(&*v,             &[101]);
    s.push_str("bar");
    v.push      (666);
});
when(&string,           |s|   assert_eq!(&*s,   "foobar"));
when(&vec,           |v|   assert_eq!(&*v,             &[101, 666]));
```

##

<!-- https://excalidraw.com/#json=-ZO3ZfUoO2w5aw8UONjJy,87x_zyNBCiW0Vqmnbm08Jg -->
![](./img/layout-1.png)

##

```rust
#[repr(C)]
struct ActualCown {
    _marker: MaybeUninit<[*const (); 4]>,
}
#[repr(C)]
struct CownDataToxic<T> {
    cown: ActualCown, // Must be first, so we can cast pointers
    data: T,
}
```


##

```cpp
typedef void (*dtor)(void*);
class DtorThunk
{
  dtor dtor_;
  DtorThunk(dtor dtor) : dtor_(dtor) {}
  ~DtorThunk();
};

cown_ptr<DtorThunk> make_boxcar_cown(size_t size, dtor dtor)
{
  ActualCown<DtorThunk>* ptr = new (size) ActualCown<DtorThunk>(dtor);
  return cown_ptr<DtorThunk>(ptr);
}
```

##

```cpp
template<class T>
class VCown : public VBase<T, Cown>
{
  // Original
  void* operator new(size_t)
  {
    return Object::register_object(
      ThreadAlloc::get().alloc<vsizeof<T>>(), VBase<T, Cown>::desc());
  }

  // My new thing
  void* operator new(size_t base_size, size_t req_size)
  {
    assert(req_size >= base_size);
    return Object::register_object(
      ThreadAlloc::get().alloc(req_size), VBase<T, Cown>::desc());
  }
};
```

## Thing's I'd love to know:

- What do you want out of this project?
- What of this can go upstream, and how?
- How will the Verona compiler interact with the runtime (embed clang??).

## Questions?

- Source: [github.com/aDotInTheVoid/boxcars](https://github.com/aDotInTheVoid/boxcars)
- Email: [alona.enraght-moony21@imperial.ac.uk](mailto:alona.enraght-moony21@imperial.ac.uk)