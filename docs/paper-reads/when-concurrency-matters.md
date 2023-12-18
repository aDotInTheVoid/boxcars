---
title: When concurrency matters
---

<https://dl.acm.org/doi/pdf/10.1145/3622852>

## 1: Intro

- Cown: owned resources
- when: introduces a behaviour.
- behaviors:
	- lock all resuoruced (deadlock-free)
	- run body
		- update resources
		- spawn behaviours
	- release resources

## 2: BoC Overview

- Underlying language must "partition heap into unique sets"
	- https://link.springer.com/chapter/10.1007/978-3-540-45070-2_9
	- This sounds like rusts unique references, but more invistigation needed
- 2 things: Cown and behaviour
- cown:
	- *aquired* or *availible*
- behaviour:
	- unit of concurrnet execution
	- list (set??) of required cowns (or resoures??)
	- have a closure

> A behaviour 𝑏 will happen before another behaviour 𝑏′ iff 𝑏 and 𝑏′ require
overlapping sets of cowns, and 𝑏 is spawned before 𝑏′.

This allows !(a happensBefore b) and !(b happpensBefore a)

- Only aquire on spawn, and release on done.
- Cowns can only be aquired by one behaviour at a time.

From AFD:

- **Race Condition**: Multiple interactions between threads that effect execution
    - Eg which thread aquires a mutex
    - Fine if they're intentional
- A **Data Race** occors when:
    - Distinct threads *access* a memory location
    - At least one of the accesses *modifies* the location
    - At least one of the accesses is *not-atomic*
    - The accesses are **not ordered* by syncronization.
- Data Races are *undefined behaviour*
- Data Races are **always** bugs, but race conditions can be fine.

- BoC is datarace and deadlock free by construction
	- But still allows race conditions:
	
```
when() {
	when (a) { /* b1 */ }
}
when ()
	when (a) { /* b2 */ }
}
```
b1 and b2 race condition, but not data race. Look at afd notes again for this.

https://doc.rust-lang.org/nomicon/races.html


> It is the remit of the underlying language to ensure that each object in the heap is owned by
exactly one entry point, and that any object accessed within the closure of a when is owned by one
of the acquired cowns.


- spawning is synronous, but running is asynronous.

> Namely, a behaviour will only be run once all behaviours that must happen before it
have been run

I need to read up on the formal model of "happens before". IIRC this is from lamport, and comes up TAPOCC



Q about listing 8: Why does b1 happenBefore b3

# 3: Semanics

A load of MoC stuff

## 4: C++ Impl

- DAG of behaviours

- Each Cown points to the last behaviour *scheduled* by that Cown.
- C++ manages lifetime of Cowns, so it's fun lifetime fun.

# 5: Evaluation

- Perf is good
- Better model than actors for some things
- Ideal Dining Philosophers

## 6: Related works

- These a load of actor langs

> OOPSLA companion paper is missing!!


