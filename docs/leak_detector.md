# Leak Detector

snmalloc/verona-rt comes with a leak detector. However it has some key limitations.

1. Global state: Once one session has leaked, all future sessions will report the same leak.
2. No information about the leak: It will tell you that there is a leak, but not where it is.

For this reason, all tests that for the leak detector
need to run in there own process. This is done with e2e tests, with just one `#[test]` per process (ie top level file in `./crates/verona-rt/tests`).