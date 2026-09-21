"""Extract text from a PDF using only the Python standard library.

No PyPDF2/pypdf available, so this does the minimum that works for text-based
PDFs: find the content streams, inflate them with zlib, and pull the text out of
the `Tj`/`TJ` operators.

Two things this gets right that an earlier version did not, both of which cost
real time:

1. **Content streams can be split between tokens.** A `[(a) -1 (b)]` array may
   close at the end of one stream while the `TJ` that shows it opens the next.
   Requiring `]` and `TJ` to be adjacent therefore drops the last line of *every*
   stream -- a silent loss in exactly the place a figure tends to sit. An array
   that runs to the end of a stream is accepted on that basis alone.

2. **A failure to inflate is not evidence of a scanned document.** The previous
   version printed "the PDF is probably image-only (needs OCR)" whenever it
   decoded no streams at all, which conflates "this file has no text" with "my
   decompressor did not run". Those need different responses, so they are
   reported separately.

It still cannot read scanned images, and it does not map Identity-H glyph ids
through a ToUnicode CMap -- see `pdf_identity_h_text.py` for that case.
"""

import re
import sys
import zlib


def inflate_streams(data):
    """Yield the decoded content of every FlateDecode stream.

    The lookbehind excludes the `stream` inside `endstream`, which otherwise
    matches as a stream opener. The trailing EOL before `endstream` is stripped
    because it is not part of the compressed data.
    """
    for match in re.finditer(rb"(?<![A-Za-z])stream\r?\n", data):
        start = match.end()
        end = data.find(b"endstream", start)
        if end == -1:
            continue
        raw = data[start:end].rstrip(b"\r\n")
        try:
            inflater = zlib.decompressobj()
            yield inflater.decompress(raw) + inflater.flush()
        except zlib.error:
            # Not a Flate stream after all.
            continue


# An array shown by `TJ`, or -- because streams may be split between tokens -- an
# array that reaches the end of its stream without one.
ARRAY = re.compile(rb"\[(.*?)\](?:\s*TJ|\s*\Z)", re.S)
STRING = re.compile(rb"\((?:\\.|[^\\()])*\)")
SINGLE = re.compile(rb"\((?:\\.|[^\\()])*\)\s*Tj")


def streams_to_text(streams):
    """Pull printable text out of PDF text-showing operators."""
    lines = []
    for content in streams:
        # Text is shown by (string) Tj and by [(a) -1 (b)] TJ arrays.
        for tj in ARRAY.finditer(content):
            body = tj.group(1)
            parts = STRING.findall(body)
            lines.append(b"".join(p[1:-1] for p in parts))
        for single in SINGLE.finditer(content):
            raw = single.group(0)
            raw = raw[: raw.rfind(b")")]
            lines.append(raw[raw.find(b"(") + 1 :])
    # Decode: PDF strings are usually Latin-1 or PDFDocEncoding for this data.
    decoded = []
    for chunk in lines:
        text = chunk.replace(b"\\(", b"(").replace(b"\\)", b")").replace(b"\\\\", b"\\")
        for enc in ("utf-8", "latin-1"):
            try:
                decoded.append(text.decode(enc))
                break
            except UnicodeDecodeError:
                continue
    return decoded


def main():
    # German legal sources are full of dashes and umlauts, and the default
    # Windows console encoding is cp1252, which raises on some of them.
    sys.stdout.reconfigure(encoding="utf-8")

    path = sys.argv[1]
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 60

    with open(path, "rb") as fh:
        data = fh.read()

    streams = list(inflate_streams(data))
    print(f"# decoded {len(streams)} content streams from {path}", file=sys.stderr)

    if not streams:
        print(
            "NO STREAMS DECODED -- every stream failed to inflate. This says "
            "nothing about whether the document has text; it usually means the "
            "file is not FlateDecode, or is encrypted.",
            file=sys.stderr,
        )
        return

    pieces = streams_to_text(streams)
    print(f"# extracted {len(pieces)} text pieces", file=sys.stderr)

    if not any(piece.strip() for piece in pieces):
        print(
            "NO TEXT OPERATORS FOUND -- the streams decoded but carried no "
            "Tj/TJ text. The file may be image-only (needs OCR) or may encode "
            "text as Identity-H glyph ids; try pdf_identity_h_text.py.",
            file=sys.stderr,
        )
        return

    shown = 0
    for piece in pieces:
        text = piece.strip()
        if not text:
            continue
        print(text)
        shown += 1
        if shown >= limit:
            print(f"... (stopped at {limit})")
            break


if __name__ == "__main__":
    main()
