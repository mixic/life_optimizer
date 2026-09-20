"""Characterise the numeric tokens in the Bern tariff PDF.

Goal: determine whether the rate column ("Einheits-Satz [%]") is recoverable.
Rates in the header sample were `1.5500` / `2.1682` -- one leading digit and
four decimals -- so if the decode is complete, tokens of that shape must exist.
"""

import importlib.util
import os
import re
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "dec", os.path.join(HERE, "decode_bern_numbers.py")
)
dec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dec)

cmap = dec.build_cmap(open(dec.PDF, "rb").read())
ops = list(dec.raw_text_operands(cmap))


def clean(text):
    for a in "'\u2019\u2018\u02bc\ufffd":
        text = text.replace(a, "")
    return text


# Collect every whitespace-separated token across all operands.
tokens = []
for op in ops:
    for piece in clean(op).split():
        piece = piece.strip()
        if piece:
            tokens.append(piece)

numeric = [t for t in tokens if re.fullmatch(r"\d+(?:\.\d+)?", t)]
print("total tokens:", len(tokens))
print("numeric tokens:", len(numeric))

shapes = Counter()
for t in numeric:
    if "." in t:
        intpart, frac = t.split(".")
        shapes[f"int{len(intpart)}_frac{len(frac)}"] += 1
    else:
        shapes[f"int{len(t)}"] += 1
print("\ntoken shapes:", dict(shapes.most_common(15)))

# Four-decimal values are the rate signature.
rates = [t for t in numeric if "." in t and len(t.split(".")[1]) == 4]
print(f"\n4-decimal tokens (rate signature): {len(rates)}")
print("  sample:", rates[:12])
if rates:
    vals = [float(r) for r in rates]
    print(f"  range: {min(vals)} .. {max(vals)}")

# Also look for 2-decimal values, which would be the tax column.
two = [t for t in numeric if "." in t and len(t.split(".")[1]) == 2]
print(f"\n2-decimal tokens (tax signature): {len(two)}")
print("  sample:", two[:12])
if two:
    vals = [float(t) for t in two]
    print(f"  range: {min(vals)} .. {max(vals)}")

ints = [int(t) for t in numeric if "." not in t]
print(f"\ninteger tokens: {len(ints)}")
print("  distinct sample:", sorted(set(ints))[:25])
