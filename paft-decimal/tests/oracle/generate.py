#!/usr/bin/env python3
"""Generate exact arithmetic cases with Python's independent Fraction oracle.

Run this script to regenerate cases.json, or pass --check to verify it. Cargo
tests consume the committed data and do not require Python at test runtime.
"""

import argparse
from fractions import Fraction
import json
from pathlib import Path
import random

MAX_COEFFICIENT = (1 << 96) - 1


def representable(value):
    """Encode exactly when the reduced rational fits a 96-bit decimal."""
    if value is None:
        return None
    denominator = value.denominator
    twos = fives = 0
    while denominator % 2 == 0:
        twos += 1
        denominator //= 2
    while denominator % 5 == 0:
        fives += 1
        denominator //= 5
    scale = max(twos, fives)
    if denominator != 1 or scale > 28:
        return None
    coefficient = value.numerator * 2 ** (scale - twos) * 5 ** (scale - fives)
    if abs(coefficient) > MAX_COEFFICIENT:
        return None
    return [str(coefficient), scale]


def cases():
    pairs = []
    # Equal-scale overflow, exact reduction, cancellation, and tiny deltas.
    for scale in [0, 1, 2, 9, 18, 27, 28]:
        for sign in [-1, 1]:
            for delta in [-10, -5, -1, 0, 1, 5, 10]:
                pairs.append(((sign * MAX_COEFFICIENT, scale), (delta, scale)))
            pairs.append(((MAX_COEFFICIENT, scale), (-MAX_COEFFICIENT, scale)))
            pairs.append(((MAX_COEFFICIENT, scale), (1, 28)))
    # Products whose unreduced coefficients exceed i128 but cancel exactly.
    for power in range(65, 96):
        for scale in [18, 27, 28]:
            pairs.append(((2 ** power, scale), (5 ** scale, 0)))
    # Terminating and nonterminating division, underflow, and signs.
    for coefficient in [0, 1, -1, 125, -125, MAX_COEFFICIENT]:
        for scale in [0, 2, 28]:
            for divisor in [0, 1, -1, 2, 3, 5, 7, 8, 10, 11, 25, 125]:
                pairs.append(((coefficient, scale), (divisor, 0)))
    rng = random.Random(0x50414654)
    for _ in range(1200):
        def operand():
            bits = rng.choice([1, 8, 32, 64, 95, 96])
            coefficient = rng.getrandbits(bits) * rng.choice([-1, 1])
            return coefficient, rng.randrange(29)
        pairs.append((operand(), operand()))
    result = []
    for (left_coefficient, left_scale), (right_coefficient, right_scale) in pairs:
        left = Fraction(left_coefficient, 10 ** left_scale)
        right = Fraction(right_coefficient, 10 ** right_scale)
        result.append({
            "lhs": [str(left_coefficient), left_scale],
            "rhs": [str(right_coefficient), right_scale],
            "add": representable(left + right),
            "sub": representable(left - right),
            "mul": representable(left * right),
            "div": representable(left / right if right else None),
        })
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    fixtures = cases()
    output = "[\n" + ",\n".join(json.dumps(case, separators=(",", ":")) for case in fixtures) + "\n]\n"
    path = Path(__file__).with_name("cases.json")
    if args.check:
        if path.read_text() != output:
            raise SystemExit("Oracle cases are stale; run generate.py")
    else:
        path.write_text(output)
    for operation in ["add", "sub", "mul", "div"]:
        successes = sum(case[operation] is not None for case in fixtures)
        print(f"{operation}: {successes} exact successes, {len(fixtures) - successes} rejections")
