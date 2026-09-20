"""Reconstruct numeric values from the Bern tariff PDF.

All glyph codes resolve through the single visible CMap, so the numbers are
recoverable.

Two formatting details that must be handled:

  * Swiss thousands separator is an apostrophe, and this PDF encodes it as
    U+2019 (RIGHT SINGLE QUOTATION MARK) rather than ASCII `'`. Treating only
    ASCII as a separator silently splits ``1'000`` into ``1`` and ``000``.
  * The decimal point is a plain dot, with four decimals for the rate column.

Numbers are emitted as they appear in stream order. The page layout is
multi-column, so this flat list is raw material, not a finished table.
"""

import re
import sys
import zlib

PDF = "Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf"

# Both the ASCII apostrophe and the typographic one are thousands separators.
APOSTROPHES = "'\u2019\u2018\u02bc"


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


def build_cmap(data):
    m = {}
    for buf in streams(data):
        if b"begincmap" not in buf:
            continue
        for block in re.findall(rb"beginbfchar(.*?)endbfchar", buf, re.S):
            for src, dst in re.findall(rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
                m[int(src, 16)] = bytes.fromhex(dst.decode()).decode("utf-16-be", "replace")
        for block in re.findall(rb"beginbfrange(.*?)endbfrange", buf, re.S):
            for g in re.finditer(rb"<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>\s*<([0-9A-Fa-f]+)>", block):
                lo, hi, base = int(g.group(1), 16), int(g.group(2), 16), int(g.group(3), 16)
                for i in range(hi - lo + 1):
                    m[lo + i] = chr(base + i)
    return m


def raw_text_operands(cmap):
    """Yield the decoded text of each TJ operand, in stream order."""
    data = open(PDF, "rb").read()
    for buf in streams(data):
        for tj in re.finditer(rb"\[(.*?)\]\s*TJ", buf, re.S):
            text = ""
            for h in re.findall(rb"<([0-9A-Fa-f]+)>", tj.group(1)):
                for i in range(0, len(h) - 3, 4):
                    text += cmap.get(int(h[i:i + 4], 16), "")
            yield text


def normalise(text):
    """Strip thousands separators and any stray non-numeric characters."""
    for a in APOSTROPHES:
        text = text.replace(a, "")
    return text.strip()


def main():
    cmap = build_cmap(open(PDF, "rb").read())

    raw_ops = list(raw_text_operands(cmap))

    # The glyph covering the thousands separator may be mapped to U+2019, to a
    # replacement char, or to nothing at all depending on which CMap wins. All
    # that matters for parsing is that it is NOT a digit or a decimal point, so
    # it is removed wherever it appears between digits. This is what previously
    # discarded every rate and tax value: the token regex rejected ``1'002.95``.
    def strip_separators(text):
        for a in APOSTROPHES:
            text = text.replace(a, "")
        text = text.replace("\ufffd", "")
        return text

    tokens = []
    for operand in raw_ops:
        cleaned = strip_separators(operand)
        for piece in re.split(r"\s+", cleaned):
            piece = piece.strip()
            if re.fullmatch(r"\d+(?:\.\d+)?", piece):
                tokens.append(piece)

    print(f"# {len(tokens)} clean numeric tokens", file=sys.stderr)

    limit = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    for t in tokens[:limit]:
        print(t)

    ints = [int(t) for t in tokens if "." not in t]
    decimals = [float(t) for t in tokens if "." in t]
    rates = [d for d in decimals if d < 20]
    amounts = [d for d in decimals if d >= 20]
    print(f"# integers: {len(ints)}  max {max(ints) if ints else 0}", file=sys.stderr)
    print(f"# rate-like (<20): {len(rates)}  range"
          f" {min(rates) if rates else 0}..{max(rates) if rates else 0}", file=sys.stderr)
    print(f"# amount-like (>=20): {len(amounts)}", file=sys.stderr)



if __name__ == "__main__":
    main()
