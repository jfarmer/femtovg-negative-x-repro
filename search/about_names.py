#!/usr/bin/env python3
"""Chooses the 14 names of the About panel (src/about-names.txt).

usage: about_names.py                      prints the names; statistics go to stderr
       about_names.py --headings           repeats the search for each version line and heading
       about_names.py --random <n> [seed]  counts for n lists drawn at random from the same pool

What varies: which names are listed and in which order. A name is any first name below with any
surname below, and no first name or surname is used twice. With --headings also the version line
and the heading above the names, from the alternatives in VERSIONS and HEADINGS. The licence
notice and the copyright line are fixed. The application name is left out: it is 20 px text, and
its glyph masks are separate from those of the 10 px text.

What is maximised: `score`, over all the 10 px lines of the panel in the order about.rs draws
them. It counts the pairs of neighbouring glyphs whose gap master draws wrong by 1 px or more,
and weights a gap inside a word that is too wide highest, because it splits the word.

How: names are added one at a time, each time the name with the highest score given the lines
above it. Then each name in turn is replaced by the name from the pool that raises the score of
the whole panel most, until no replacement raises it. Nothing here is random.

The pool was written for this search and has many names with i, l and t in them. Those letters
are narrow, so a 1 px error is large next to them.
"""
import random
import sys

from replay import ROBOTO_FLEX, gaps, measure, prepare

FONT_SIZE = 10
NAMES = 14

HEAD = ["Version 1.11 (1111)", "Developed by"]
TAIL = [
    "This program is free software; you can redistribute it and/or",
    "modify it under the terms of the GNU General Public License.",
    "It is distributed in the hope that it will be useful, but",
    "without any warranty.",
    "© 2026 The Carillon Contributors",
]
VERSIONS = ["Version 1.11.1 (build 1111)", "Version 1.1.11 (build 111)", "Version 11.1 (build 1111)",
            "Version 1.11.1", "Version 1.11 (1111)", "Version 1.11.1 (11 July 2026)"]
HEADINGS = ["Contributors", "Credits", "Written by", "With thanks to", "Developed by"]

FIRST = """Lillian William Emilia Millie Philip Julia Tim Nikhil Li Illia Finn Kirill Vinh Ilse Willem Hillel
Mihail Emil Tilly Colin Kimi Linh Jill Milan Dmitri Viktor Lucille Camille Felix Nils Till Alina Lidia
Elliot Hillary Gillian Timothy Dimitri Titus Minh Phil Liam Niall Aili Ingrid Ilya Irina Iris Ivan Isla
Imani Ruth Arthur Martin Bertil Hattie Mattie Judith Edith Keith Willa Lila Mila Nina Tina Rina Anil
Anna Maria Daniel Thomas Henrik Mikhail Olivia Elif Luis Luigi Fatima Benjamin Christine Patricia
Vincent Cecilia Miriam Kristin Antti Sofia Lucia Emma Oliver Hannah""".split()
LAST = """Hill Mitchell Tillman Whitfield Little Villiers Miller Malik Lin Shymko Hillier Filippov Lim
Illingworth Whittle Tillich Phillips Williams Millington Fielding Kimball Hilliard Lindqvist Nilsson
Linnell Pilkington Tuttle Liddell Finnigan Mullin Hutchins Dillon Fitch Litvin Minelli Bellini Tintoretto
Fillmore Gilliam Billings Twining Willis Ellis Inglis Wills Mills Tully Lilly Kelly Hallett Pruitt Truitt
Whitlock Kittredge Littlefield Middleton Tillotson Villanueva Guillaume Mirzoyan Trinh
Nguyen Smith Fischer Schmidt Bianchi Rinaldi Virtanen Laitinen Niemi Liu Kim Singh Silva Hamilton
Clifton Elliott Griffin Griffith Sullivan Collins""".split()
POOL = [f"{first} {last}" for first in FIRST for last in LAST]


def prepared(lines):
    return [prepare(glyphs) for glyphs in measure(ROBOTO_FLEX, FONT_SIZE, [], lines)]


def score(pairs):
    total = 0.0
    for gap, across_space in pairs:
        if across_space:
            total += 1.0 if abs(gap) >= 1.0 else 0.0
        elif gap >= 1.0:
            total += 1.5
        elif gap <= -1.0:
            total += 1.0
        elif abs(gap) >= 0.75:
            total += 0.25
    return total


def panel_pairs(lines):
    """The pairs of all the lines, drawn in order with one cache."""
    cache, pairs = {}, []
    for line in lines:
        pairs += gaps(line, cache)
    return pairs


def usable(name, chosen, skip=None):
    """Whether neither part of `name` is in the names already chosen, ignoring `chosen[skip]`."""
    first, last = name.split()
    return not any(i != skip and (first in other.split() or last in other.split())
                   for i, other in enumerate(chosen))


def search(head, pool, tail):
    """`head` and `tail` are prepared lines, `pool` maps a name to its prepared line.
    Returns (score, names)."""
    cache = {}
    for line in head:
        gaps(line, cache)
    chosen = []
    for _ in range(NAMES):
        best = None
        for name in pool:
            if not usable(name, chosen):
                continue
            trial = dict(cache)
            value = score(gaps(pool[name], trial))
            if best is None or value > best[0]:
                best = (value, name, trial)
        chosen.append(best[1])
        cache = best[2]

    best = score(panel_pairs(head + [pool[name] for name in chosen] + tail))
    improved = True
    while improved:
        improved = False
        for i in range(NAMES):
            cache, before = {}, 0.0
            for line in head + [pool[name] for name in chosen[:i]]:
                before += score(gaps(line, cache))
            after = [pool[name] for name in chosen[i + 1:]] + tail
            for name in pool:
                if name == chosen[i] or not usable(name, chosen, skip=i):
                    continue
                trial = dict(cache)
                value = before + score(gaps(pool[name], trial))
                for line in after:
                    value += score(gaps(line, trial))
                if value > best:
                    best, chosen[i], improved = value, name, True
        print(f"score {best}", file=sys.stderr)
    return best, chosen


def counts(pairs):
    inside = [gap for gap, across_space in pairs if not across_space]
    return (f"{sum(abs(gap) >= 1 for gap in inside)} of {len(inside)} pairs with no space between and "
            f"{sum(abs(gap) >= 1 for gap, _ in pairs)} of {len(pairs)} pairs in all have a gap wrong by 1 px or more")


def main():
    pool = dict(zip(POOL, prepared(POOL)))
    tail = prepared(TAIL)
    if sys.argv[1:2] == ["--headings"]:
        for version in VERSIONS:
            for heading in HEADINGS:
                value, names = search(prepared([version, heading]), pool, tail)
                print(f"{value}\t{version}\t{heading}\t{', '.join(names)}", flush=True)
    elif sys.argv[1:2] == ["--random"]:
        rng = random.Random(int(sys.argv[3]) if len(sys.argv) > 3 else 1)
        head, inside, in_all = prepared(HEAD), [], []
        for _ in range(int(sys.argv[2])):
            names = []
            while len(names) < NAMES:
                name = rng.choice(POOL)
                if usable(name, names):
                    names.append(name)
            pairs = panel_pairs(head + [pool[name] for name in names] + tail)
            inside.append(sum(abs(gap) >= 1 for gap, across_space in pairs if not across_space))
            in_all.append(sum(abs(gap) >= 1 for gap, _ in pairs))
        for label, values in (("pairs with no space between", inside), ("all pairs", in_all)):
            values.sort()
            print(f"{label} with a gap wrong by 1 px or more, over {len(values)} random lists: "
                  f"mean {sum(values) / len(values):.1f}, median {values[len(values) // 2]}, "
                  f"smallest {values[0]}, largest {values[-1]}")
    else:
        head = prepared(HEAD)
        value, names = search(head, pool, tail)
        print(f"score {value}: {counts(panel_pairs(head + [pool[name] for name in names] + tail))}", file=sys.stderr)
        print("\n".join(names))


if __name__ == "__main__":
    main()
