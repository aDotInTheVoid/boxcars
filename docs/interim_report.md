---
title: "BoC in Rust: Interim Report"
author: Alona Enraght-Moony
date: 2024-01-11
bibliography: cites.bib
csl: https://raw.githubusercontent.com/citation-style-language/styles/master/vancouver.csl
link-citations: true
---

## Background

### Rust

Rust [@rust_book] is a programming languages originally developed by Mozilla Research,
and currently maintained by a large cross-org team.

### Behaviour-Oriented Concurrency

Behaviour-Oriented Concurrency (BOC) is a novel concurrency paradime [@when_concurrency_matters]

### Other concurrency paradimes

#### Shared Memory

#### Message Passing

#### Fork/Join

#### Actors

#### Structured Concurrency

### Verona Runtime

## The work done so far

## Ethical Issues

This project presents very few ethical issues. The only thing to be concerted
about is clear attribution of work, as it builds extensively on pre-existing
research into Behavior Oriented Concurrency, and it's implementation in the
Verona Runtime. It must be clear what of the work was my own, as what wasn't.

This is made more complex by the fact that I have a direct dependency on the
verona-rt library, and have made modification to it to support my specific use
case. I'll also (hopefully) end up conversing with MSR people about how to integrate these changes upstream, and other possible directions. I need to be careful to properly acknowledge this potential collaboration (should it occur).

## Bibliography