# Memory Layout.

It's usefull to understand how verona-rt lays thing out in memory.


```c++
class cown_ptr_base {}
```

Has no fields, used for template shenanigans.

```c++
// namespace verona::rt
class Object {
  ...
};
class Shared : public Object {
  ...
};
class Cown : public Shared {
  ...
};

// Adds no fields, but adds methods using `T` to `Base`
template <class T, class Base = Object>
class VBase : public Base {};

// Has no fields, but adds methods using `T` to `Cown`
template <class T>
class VCown : public VBase<T, Cown> {};

template <typename T>
class ActualCown : public VCown<ActualCown<T>> {
 private:
  T value;
};
```

```c++
template <typename T>
class cown_ptr {
    ActualCown<T>* ptr;
}
```

```c++
class CownMeta: VCown<CownMeta> {
    dtor dtor_;
};
```



