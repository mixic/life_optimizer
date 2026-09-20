"""Decode the Bern tariff PDF's numeric table using its own ToUnicode CMaps.

The document is vector text in Identity-H fonts. One embedded CMap is fully
visible (digits 0-9, '.', and a few others); the others live inside compressed
object streams. This script:

  1. collects every CMap it can find,
  2. reports how many glyph codes each one covers,
  3. decodes TJ hex operands and reports how many resolve to digits,

so we can tell the difference between "the data is not there" and "we cannot
read it yet".
"""

import re
import sys
import zlib
from collections import Counter

PDF = "Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf"


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


def parse_cmap(buf):
    m = {}
    for block in re.findall(rb"beginbfchar(.*?)endbfchar", buf, re.S):
        for src, dst in re.findall(rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
            m[int(src, 16)] = bytes.fromhex(dst.decode()).decode("utf-16-be", "replace")
    for block in re.findall(rb"beginbfrange(.*?)endbfrange", buf, re.S):
        for g in re.finditer(rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
            lo, hi, base = int(g.group(1), 16), int(g.group(2), 16), int(g.group(3), 16)
            for i in range(hi - lo + 1):
                m[lo + i] = chr(base + i)
    return m


data = open(PDF, "rb").read()
all_streams = list(streams(data))
print("decoded streams:", len(all_streams))

merged = {}
cmap_count = 0
for buf in all_streams:
    if b"begincmap" in buf:
        cmap_count += 1
        merged.update(parse_cmap(buf))
print("CMap streams with begincmap:", cmap_count)
print("merged glyph mappings:", len(merged))
print("mapped glyphs sample:", sorted((f"0x{k:04X}", v) for k, v in list(merged.items())[:20]))

# How many TJ hex operands are there in total, and how long are they?
hex_ops = []
for buf in all_streams:
    for tj in re.finditer(rb"\[(.*?)\]\s*TJ", buf, re.S):
        hex_ops.extend(re.findall(rb"<([0-9A-Fa-f]+)>", tj.group(1)))
print("total TJ hex operands:", len(hex_ops))

total_glyphs = 0
resolved = 0
digit_glyphs = 0
code_hist = Counter()
for h in hex_ops:
    for i in range(0, len(h) - 3, 4):
        code = int(h[i:i + 4], 16)
        total_glyphs += 1
        code_hist[code] += 1
        if code in merged:
            resolved += 1
            if merged[code].isdigit():
                digit_glyphs += 1

print("total glyph codes:", total_glyphs)
print("resolved via CMap:", resolved, f"({resolved/max(1,total_glyphs)*100:.1f}%)")
print("of which digits:", digit_glyphs)
print("most common codes:", code_hist.most_common(12))
