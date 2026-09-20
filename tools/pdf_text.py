"""Extract text from a PDF using only the Python standard library.

No PyPDF2/pypdf available, so this does the minimum that works for
text-based PDFs: find the content streams, inflate them with zlib, and pull
the text out of the `Tj`/`TJ` operators. It will not handle every PDF, and it
cannot read scanned images -- if the output is empty, the file is likely
image-only and needs OCR.
"""

import re
import sys
import zlib


def inflate_streams(data):
    """Yield the decoded content of every FlateDecode stream."""
    out = []
    for match in re.finditer(rb"stream\r?\n", data):
        start = match.end()
        end = data.find(b"endstream", start)
        if end == -1:
            continue
        raw = data[start:end]
        try:
            out.append(zlib.decompress(raw))
        except zlib.error:
            # Not a Flate stream, or truncated.
            continue
    return out


def streams_to_text(streams):
    """Pull printable text out of PDF text-showing operators."""
    lines = []
    for content in streams:
        # Text is shown by (string) Tj and by [(a) -1 (b)] TJ arrays.
        for tj in re.finditer(rb"\[(.*?)\]\s*TJ", content, re.S):
            body = tj.group(1)
            parts = re.findall(rb"\((?:\\.|[^\\()])*\)", body)
            text = b"".join(p[1:-1] for p in parts)
            lines.append(text)
        for single in re.finditer(rb"\((?:\\.|[^\\()])*\)\s*Tj", content):
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
    path = sys.argv[1]
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 60

    with open(path, "rb") as fh:
        data = fh.read()

    streams = inflate_streams(data)
    print(f"# decoded {len(streams)} content streams from {path}", file=sys.stderr)
    if not streams:
        print("NO TEXT STREAMS -- the PDF is probably image-only (needs OCR)",
              file=sys.stderr)
        return

    pieces = streams_to_text(streams)
    print(f"# extracted {len(pieces)} text pieces", file=sys.stderr)

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
