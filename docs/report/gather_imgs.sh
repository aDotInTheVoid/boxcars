#!/bin/bash

set -eoxu pipefail

cd "$(dirname "$0")"

criterion_dir=../../target/criterion/



for bname in \
    'Busy Loop' \
    'Create Cowns' \
    'Fibonacci' \
    'Schedule Behaviours' \
    'Philosophers'
do
    cp "$criterion_dir/$bname/report/lines.svg" "./bench_graphs/${bname// /_}.svg"
done

for bname in \
	'savina_Banking' \
	'savina_Barber' \
    'Scheduler'
do
	cp "$criterion_dir/$bname/report/violin.svg" "./bench_graphs/${bname// / /_}.svg"
done

