"""Reconstruct lines from an Identity-H PDF whose text is one glyph per operator.

The DBG (SR 642.11) emits `<xxxx> Tj` per *character*, with positioning via
`Td`/`Tm`. Decoding the glyphs is not enough: the character stream has to be
re-assembled into lines using the position operators, or every letter lands on
its own line.

Strategy: walk the content stream in order, tracking the current text matrix's
Y coordinate. A change in Y (beyond a small tolerance) starts a new line; glyph
widths are approximated by accumulating an x offset and breaking on large gaps.
"""

import re
import sys
import zlib

# Reuse the glyph decoding from the identity-H helper.
import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "ih", os.path.join(HERE, "pdf_identity_h_text.py")
)
ih = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ih)


def build_cmap(data):
    merged = {}
    for buf in ih.streams(data):
        if b"beginbfchar" not in buf and b"beginbfrange" not in buf:
            continue
        for code, ch in ih.parse_cmap(buf).items():
            merged.setdefault(code, ch)
    return merged


TOKEN = re.compile(
    rb"(?P<hex><[0-9A-Fa-f]+>\s*Tj)"
    rb"|(?P<td>-?[\d.]+)\s+(?P<tdy>-?[\d.]+)\s+Td"
    rb"|(?P<tm>(?P<a>-?[\d.]+)\s+(?P<b>-?[\d.]+)\s+(?P<c>-?[\d.]+)\s+"
    rb"(?P<d>-?[\d.]+)\s+(?P<e>-?[\d.]+)\s+(?P<f>-?[\d.]+)\s+Tm)"
    rb"|(?P<bt>\bBT\b)"
)


def extract_lines(path, y_tolerance=1.5):
    data = open(path, "rb").read()
    cmap = build_cmap(data)
    out_lines = []

    for buf in ih.streams(data):
        if b"Tj" not in buf:
            continue

        line = []
        last_y = None
        x = 0.0
        prev_x_end = None

        for m in TOKEN.finditer(buf):
            if m.group("td") is not None:
                x = float(m.group("td"))
                y = float(m.group("tdy"))
            elif m.group("tm") is not None:
                x = float(m.group("e"))
                y = float(m.group("f"))
            elif m.group("bt") is not None:
                last_y = None
                continue
            else:
                # A glyph: place it on the current line, breaking if the
                # vertical position moved.
                if last_y is None:
                    last_y = y
                elif abs(y - last_y) > y_tolerance:
                    if line:
                        out_lines.append("".join(line))
                    line = []
                    last_y = y

                hexs = re.search(rb"<([0-9A-Fa-f]+)>", m.group("hex")).group(1)
                text = ""
                for i in range(0, len(hexs) - 3, 4):
                    text += cmap.get(int(hexs[i:i + 4], 16), "")
                # A big horizontal jump means a column gap.
                if prev_x_end is not None and x - prev_x_end > 12.0 and line:
                    line.append("  ")
                line.append(text)
                prev_x_end = x
                continue
            prev_x_end = None

        if line:
            out_lines.append("".join(line))

    return out_lines


def main():
    path = sys.argv[1]
    out_path = sys.argv[2]

    lines = extract_lines(path)
    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))

    print(f"lines extracted: {len(lines)}")
    print(f"written        : {out_path} ({os.path.getsize(out_path)} bytes)")
    print("--- first 14 lines ---")
    for ln in lines[:14]:
        print("   ", ln[:110])


if __name__ == "__main__":
    main()
