#!/usr/bin/env python

from pathlib import Path
import os
from pprint import pprint

import matplotlib.pyplot as plt
import matplotlib.ticker as tck
import numpy as np
import json

BENCH_BASE = Path("../bench/09284ec/")


def load_mean(base, impl, input):
    with open(base / impl / input / "new" / "estimates.json") as f:
        return json.load(f)["mean"]["point_estimate"]


def plot_nice_bar(name, base_impl, xlabel, filename):
    # https://matplotlib.org/stable/gallery/lines_bars_and_markers/barchart.html

    this_base = BENCH_BASE / name

    # impls = keys(penguine_species)
    impls = sorted(os.listdir(this_base))
    # sizes = species
    sizes = sorted(os.listdir(this_base / impls[0]), key=int)

    x = np.arange(len(sizes))
    width = 0.8 / len(impls)
    multiplier = 0

    fig, ax = plt.subplots()

    norm_to = {size: load_mean(this_base, base_impl, size) for size in sizes}

    for impl in impls:
        offset = width * multiplier

        data = [load_mean(this_base, impl, size) / norm_to[size] for size in sizes]

        rect = ax.bar(x + offset, data, width, label=impl)
        ax.bar_label(rect, padding=3, rotation=90, label_type="center", fmt="%.3f")
        multiplier += 1

    ax.set_ylabel(f"Time (relative to {base_impl})")
    ax.set_xlabel(xlabel)
    ax.set_xticks(x + width, sizes)

    ax.yaxis.set_minor_locator(tck.AutoMinorLocator())

    ax.legend(loc="lower left")

    fig.savefig(filename)
    print(f"wrote {filename}")
    plt.close(fig)


if __name__ == "__main__":
    plot_nice_bar(
        name="Create Cowns",
        base_impl="verona-rt",
        xlabel="Number of Cowns",
        filename="create_cown.pdf",
    )
    # TODO: Busyloop
    plot_nice_bar(
        name="Schedule Behaviours",
        base_impl="verona-rt",
        xlabel="Number of Behaviours",
        filename="schedule_behaviours.pdf",
    )

    plot_nice_bar(
        name="Fibonacci",
        base_impl="careful verona-rt",
        xlabel="fib(n) being calculated",
        filename="fibonacci.pdf",
    )
