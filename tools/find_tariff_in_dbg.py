"""Search the decoded DBG text for the Art. 36 tariff.

Line reconstruction is imperfect (the PDF writes one glyph per operator), so
this searches the flat character stream for the tariff's distinctive values and
prints a window around any hit, rather than relying on line breaks.
"""

import re

text = open("sr642.txt", encoding="utf-8").read()
# Re-join everything: the stream is one character per piece.
flat = "".join(text.split("\n"))

# Art. 36's single tariff opens with a 0% band and 0.77% first rate.
needles = ["0,77", "0.77", "14500", "14 500", "1,77", "0,11", "Grundtarif", "Art.36", "Art. 36"]
for needle in needles:
    idxs = [m.start() for m in re.finditer(re.escape(needle), flat)]
    print(f"{needle!r}: {len(idxs)} occurrence(s)")
    for start in idxs[:2]:
        window = flat[max(0, start - 160): start + 260]
        out = window if isinstance(window, str) else str(window)
        with open("tariff_window.txt", "a", encoding="utf-8") as fh:
            fh.write(f"\n===== {needle!r} at {start} =====\n{out}\n")

print("\nwindows written to tariff_window.txt")
