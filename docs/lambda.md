# Lambda Investigation.

1. `lambdabehaviour.h` `schedule_lambda(size_t count, Cown** cowns, T&& f)`

    It's a thin wrapper over:

2. `behaviour.h` `static void schedule(size_t count, Cown** cowns, T&& f)`

    Which does a alloc :( to build an array of requests and then call into:

3. `behaviour.h` `schedule(size_t count, Request* requests, T&& f)`.

    This is where the rubber meets the road.

    ```cpp
    // This is where we eraise things.
    Behaviour* body =
      prepare_to_schedule<T>(count, requests, std::forward<T>(f));
    
    // None of this is templated, so we can just call it.
    BehaviourCore* arr[] = {body};
    BehaviourCore::schedule_many(arr, 1);
    ```

4. `behaviour.h` `prepare_to_schedule(size_t count, Request* requests, T&& f)`


    `Behaviour` isn't generic here, but `prepare_to_schedule` is.

    ```cpp
    template<typename T>
    static Behaviour*
    prepare_to_schedule(size_t count, Request* requests, T&& f)
    {
      auto body = Behaviour::make<T>(count, std::forward<T>(f));

      auto* slots = body->get_slots();
      for (size_t i = 0; i < count; i++)
      {
        auto* s = new (&slots[i]) Slot(requests[i].cown());
        if (requests[i].is_move())
          s->set_move();
      }

      return body;
    }
    ```

    The templatedness is pushed into `Behaviour::make`, the rest is bookkeeping.

5. `Behaviour::make`

    ```cpp
    template<typename T>
    static Behaviour* make(size_t count, T&& f)
    {
      auto behaviour_core = BehaviourCore::make(count, invoke<T>, sizeof(T));

      new (behaviour_core->get_body()) T(std::forward<T>(f));

      // These assertions are basically checking that we won't break any
      // alignment assumptions on T.  If we add some actual alignment, then
      // this can be improved.
      static_assert(
        alignof(T) <= sizeof(void*), "Alignment not supported, yet!");

      return (Behaviour*)behaviour_core;
    }
    ```

6. `BehaviourCore::make`:

    `static BehaviourCore* make(size_t count, void (*f)(Work*), size_t payload)`.