#!/bin/bash

set -eoxu pipefail

cd "$(dirname "$0")"

criterion_dir=../../target/criterion/



for bname in \
    'Busy Loop' \
    'Create Cowns' \
    'Fibonacci' \
    'Schedule Behaviours'
do
    cp "$criterion_dir/$bname/report/lines.svg" "./img/${bname// /_}.svg"
done

for bname in \
	'savina_Banking' \
	'savina_Barber'
do
	cp "$criterion_dir/$bname/report/violin.svg" "./img/${bname// / /_}.svg"
done

