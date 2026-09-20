"""Extract PDF text to a UTF-8 file.

Same decompression approach as `pdf_text.py`, but writes to a file with an
explicit UTF-8 encoding. Printing to a Windows console fails with
`UnicodeEncodeError: 'charmap' codec can't encode character` because the console
codepage is cp1252 and the documents contain typographic dashes and accented
characters, so console output is not a reliable channel for this content.
"""

import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("pt", os.path.join(HERE, "pdf_text.py"))
pt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pt)

src = sys.argv[1]
out = sys.argv[2]

data = open(src, "rb").read()
streams = pt.inflate_streams(data)
pieces = pt.streams_to_text(streams)
text = "\n".join(p.strip() for p in pieces if p.strip())

with open(out, "w", encoding="utf-8", newline="\n") as fh:
    fh.write(text)

print(f"streams decoded : {len(streams)}")
print(f"text pieces     : {len(pieces)}")
print(f"written         : {out} ({os.path.getsize(out)} bytes)")
