"""Decode Identity-H PDF text using every ToUnicode CMap in the file.

`pdf_text.py` handles parenthesised `Tj`/`TJ` operands; this variant handles the
other common shape, where text is written as hex glyph codes (`<0041> Tj`) under
Identity-H fonts, so the codes must be resolved through the fonts' ToUnicode
CMaps.

The DBG (SR 642.11) uses this shape. Merging all CMaps is a simplification —
strictly each font has its own — so the script reports how many code collisions
it had to resolve, because a collision means some characters may be wrong.
"""

import os
import re
import sys
import zlib


def streams(data):
    for m in re.finditer(rb"stream\r?\n", data):
        s = m.end()
        e = data.find(b"endstream", s)
        if e == -1:
            continue
        try:
            yield zlib.decompress(data[s:e])
        except zlib.error:
            continue


def decode_dst(hexbytes):
    try:
        raw = bytes.fromhex(hexbytes.decode())
    except ValueError:
        return ""
    for enc in ("utf-16-be", "latin-1"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            continue
    return ""


def parse_cmap(buf):
    m = {}
    for block in re.findall(rb"beginbfchar(.*?)endbfchar", buf, re.S):
        for src, dst in re.findall(rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
            m[int(src, 16)] = decode_dst(dst)
    for block in re.findall(rb"beginbfrange(.*?)endbfrange", buf, re.S):
        for g in re.finditer(
            rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block
        ):
            lo, hi, base = int(g.group(1), 16), int(g.group(2), 16), int(g.group(3), 16)
            base_bytes = g.group(3)
            for i in range(hi - lo + 1):
                # Increment the final code unit for ranges.
                try:
                    val = bytes.fromhex(base_bytes.decode())
                    as_int = int.from_bytes(val, "big") + i
                    width = len(val)
                    m[lo + i] = as_int.to_bytes(width, "big").decode("utf-16-be", "replace")
                except Exception:
                    pass
    return m


def main():
    path = sys.argv[1]
    out_path = sys.argv[2]

    data = open(path, "rb").read()

    merged = {}
    collisions = 0
    cmap_count = 0
    for buf in streams(data):
        if b"beginbfchar" not in buf and b"beginbfrange" not in buf:
            continue
        cmap_count += 1
        for code, ch in parse_cmap(buf).items():
            if code in merged and merged[code] != ch:
                collisions += 1
                continue
            merged[code] = ch

    pieces = []
    for buf in streams(data):
        if b"Tj" not in buf and b"TJ" not in buf:
            continue
        for m in re.finditer(rb"<([0-9A-Fa-f]+)>\s*Tj", buf):
            h = m.group(1)
            text = ""
            for i in range(0, len(h) - 3, 4):
                text += merged.get(int(h[i:i + 4], 16), "")
            if text:
                pieces.append(text)
        for m in re.finditer(rb"\[(.*?)\]\s*TJ", buf, re.S):
            text = ""
            for h in re.findall(rb"<([0-9A-Fa-f]+)>", m.group(1)):
                for i in range(0, len(h) - 3, 4):
                    text += merged.get(int(h[i:i + 4], 16), "")
            if text:
                pieces.append(text)

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(pieces))

    print(f"CMaps merged : {cmap_count}")
    print(f"glyph codes  : {len(merged)}")
    print(f"collisions   : {collisions} (codes mapped differently across fonts)")
    print(f"text pieces  : {len(pieces)}")
    print(f"written      : {out_path} ({os.path.getsize(out_path)} bytes)")


if __name__ == "__main__":
    main()
