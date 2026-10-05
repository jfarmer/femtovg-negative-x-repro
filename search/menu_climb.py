#!/usr/bin/env python3
"""Chooses the dishes of the menu card (src/menu.txt).

usage: menu_climb.py [rounds] [seed] [restarts]   prints the menu; progress goes to stderr
       menu_climb.py --random <n> [seed]          counts for n menus drawn at random

The defaults, 80 rounds, seed 101 and 12 restarts, are the run that produced src/menu.txt.

What varies, all of it arbitrary on a real menu:
  - which four of the five dishes of a course are listed, and in which order;
  - the price of a dish, within 2 of its base price;
  - the order of the ingredients in the description of a dish.
The title, the line below it and the course headings are fixed. They are in other sizes or
weights than the dishes, so their glyph masks are separate and they do not enter the search.

What is maximised: `objective`, over the dish and description lines in the order menu.rs draws
them. It counts the pairs of neighbouring glyphs whose gap master draws wrong: one point for
1 px or more, two more for 1.25 px or more, three more for 1.5 px or more.

How: a hill climb. It starts from the best of 150 random menus. In each round it makes 60
menus that each differ from the current one in one random change, and moves to the best of
them unless that one is worse than the current menu. The best menu over the restarts is
printed.

A random menu, as --random draws it, has four random dishes per course in random order, with
the base prices and the descriptions as written below.
"""
import copy
import os
import random
import sys

from replay import gaps, measure, prepare

FONT = os.path.join(os.environ.get("NEW_YORK_DIR", "/System/Library/Fonts"), "NewYork.ttf")
FONT_SIZE = 12
VARIATIONS = ["opsz=12"]

# (dish, ingredients, base price) for each course.
COURSES = [
    ("ANTIPASTI", [
        ("Bruschetta al Pomodoro", "grilled ciabatta, vine tomatoes, basil, olive oil", 9),
        ("Burrata con Fichi", "figs, wild rocket, chilli, mint", 12),
        ("Vitello Tonnato", "chilled veal, tuna and caper sauce, lemon", 13),
        ("Zucchini Fritti", "courgette fritters, lemon, chilli salt", 8),
        ("Insalata di Finocchi", "fennel, blood orange, mint, pine nuts", 10),
    ]),
    ("PRIMI", [
        ("Tagliatelle al Limone", "lemon, chilli, parsley, pecorino", 14),
        ("Linguine alle Vongole", "clams, garlic, chilli, white wine", 17),
        ("Tortellini in Brodo", "veal tortellini, clear chicken broth, parmesan", 15),
        ("Risotto alla Milanese", "saffron, bone marrow, white wine, parmesan", 16),
        ("Trofie al Pesto Trapanese", "almonds, tomato, basil, garlic", 14),
    ]),
    ("SECONDI", [
        ("Pollo alla Diavola", "grilled chilli chicken, lemon, wilted spinach", 19),
        ("Filetto di Manzo", "fillet of beef, truffle butter, grilled polenta", 31),
        ("Cotoletta alla Milanese", "veal cutlet, wild rocket, lemon", 24),
        ("Branzino alla Griglia", "grilled sea bass, fennel, dill, olive oil", 26),
        ("Melanzane alla Parmigiana", "aubergine, tomato, mozzarella, basil", 17),
    ]),
    ("DOLCI", [
        ("Tiramisù", "mascarpone, espresso, marsala, cocoa", 8),
        ("Millefoglie alla Vaniglia", "puff pastry, vanilla cream, wild strawberries", 9),
        ("Panna Cotta", "vanilla, lemon, pistachio", 7),
        ("Affogato al Caffè", "vanilla gelato, espresso, amaretti", 6),
        ("Cannoli Siciliani", "ricotta, candied citrus, pistachio", 8),
    ]),
]


def random_menu(rng):
    """A menu is, per course: the indices of the four dishes listed, in order; the change of
    each dish's price; and the order of each dish's ingredients."""
    return {
        "order": [rng.sample(range(5), 4) for _ in COURSES],
        "price": [[0] * 5 for _ in COURSES],
        "ingredients": [[list(range(len(dish[1].split(", ")))) for dish in dishes] for _, dishes in COURSES],
    }


def changed(menu, rng):
    """A copy of `menu` with one random change."""
    new = copy.deepcopy(menu)
    c = rng.randrange(len(COURSES))
    kind = rng.random()
    if kind < 0.35:
        order = new["order"][c]
        if rng.random() < 0.5:
            i, j = rng.sample(range(4), 2)
            order[i], order[j] = order[j], order[i]
        else:
            order[rng.randrange(4)] = next(i for i in range(5) if i not in order)
    elif kind < 0.7:
        d = rng.choice(new["order"][c])
        new["price"][c][d] = rng.choice([v for v in (-2, -1, 0, 1, 2) if v != new["price"][c][d]])
    else:
        d = rng.choice(new["order"][c])
        rng.shuffle(new["ingredients"][c][d])
    return new


def entries(menu):
    """(course heading, [(dish, description, price)]) as listed on `menu`."""
    out = []
    for c, (heading, dishes) in enumerate(COURSES):
        listed = []
        for d in menu["order"][c]:
            name, description, price = dishes[d]
            ingredients = description.split(", ")
            listed.append((name, ", ".join(ingredients[k] for k in menu["ingredients"][c][d]), price + menu["price"][c][d]))
        out.append((heading, listed))
    return out


_prepared = {}


def pairs(menu):
    """(gap error, across a space) for the dish and description lines of `menu`, drawn in
    order with one cache."""
    lines = []
    for _, listed in entries(menu):
        for name, description, price in listed:
            lines += [f"{name} · {price}", description]
    new = [line for line in dict.fromkeys(lines) if line not in _prepared]
    if new:
        for line, glyphs in zip(new, measure(FONT, FONT_SIZE, VARIATIONS, new)):
            _prepared[line] = prepare(glyphs)
    cache, out = {}, []
    for line in lines:
        out += gaps(_prepared[line], cache)
    return out


def objective(menu):
    total = 0
    for gap, _ in pairs(menu):
        size = abs(gap)
        total += (size >= 1.0) + 2 * (size >= 1.25) + 3 * (size >= 1.5)
    return total


def climb(rounds, rng):
    best, best_value = None, -1
    for menu in [random_menu(rng) for _ in range(150)]:
        value = objective(menu)
        if value > best_value:
            best, best_value = menu, value
    for _ in range(rounds):
        candidate, candidate_value = None, -1
        for menu in [changed(best, rng) for _ in range(60)]:
            value = objective(menu)
            if value > candidate_value:
                candidate, candidate_value = menu, value
        if candidate_value >= best_value:
            best, best_value = candidate, candidate_value
    return best, best_value


def main():
    if sys.argv[1:2] == ["--random"]:
        rng = random.Random(int(sys.argv[3]) if len(sys.argv) > 3 else 1)
        values = sorted(sum(abs(gap) >= 1 for gap, _ in pairs(random_menu(rng))) for _ in range(int(sys.argv[2])))
        print(f"pairs with a gap wrong by 1 px or more in the dish and description lines, over {len(values)} "
              f"random menus: mean {sum(values) / len(values):.1f}, median {values[len(values) // 2]}, "
              f"smallest {values[0]}, largest {values[-1]}")
        return
    rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 80
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 101)
    restarts = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    best, best_value = None, -1
    for restart in range(restarts):
        menu, value = climb(rounds, rng)
        print(f"restart {restart}: objective {value}", file=sys.stderr)
        if value > best_value:
            best, best_value = menu, value
    wrong = sum(abs(gap) >= 1 for gap, _ in pairs(best))
    print(f"objective {best_value}: {wrong} of {len(pairs(best))} pairs have a gap wrong by 1 px or more", file=sys.stderr)
    print("\n\n".join(
        "\n".join([heading] + [f"{name} | {description} | {price}" for name, description, price in listed])
        for heading, listed in entries(best)))


if __name__ == "__main__":
    main()
