"""Prepend the GPL-3.0 license header to Rust source files.

The header text matches the project's LICENSE (GNU GPL v3). It is emitted as
`//` line comments rather than `//!` doc comments on purpose:

  * `//!` is a *doc comment* and must stay contiguous with any following `//!`
    block. Using `//` keeps the header independent of whatever module
    documentation already exists, which every file in this project has.
  * A blank line is inserted after the header so existing `#![...]` inner
    attributes and `//!` module docs keep their positions and semantics.

Idempotent: a file that already contains the copyright line is skipped.
"""

import os
import sys

HEADER_LINES = [
    "// Life Optimizer",
    "// Copyright (C) 2026 MILAN NIKOLIC",
    "//",
    "// This program is free software: you can redistribute it and/or modify",
    "// it under the terms of the GNU General Public License as published by",
    "// the Free Software Foundation, either version 3 of the License, or",
    "// (at your option) any later version.",
    "//",
    "// This program is distributed in the hope that it will be useful,",
    "// but WITHOUT ANY WARRANTY; without even the implied warranty of",
    "// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the",
    "// GNU General Public License for more details.",
    "//",
    "// You should have received a copy of the GNU General Public License",
    "// along with this program.  If not, see <https://www.gnu.org/licenses/>.",
]

MARKER = "Copyright (C) 2026 MILAN NIKOLIC"
HEADER = "\n".join(HEADER_LINES) + "\n\n"


def find_rust_files(root):
    for dirpath, dirnames, filenames in os.walk(root):
        # Never touch build output or VCS metadata.
        dirnames[:] = [
            d for d in dirnames
            if d not in {"target", ".git", ".dsh", "__pycache__"}
        ]
        for name in sorted(filenames):
            if name.endswith(".rs"):
                yield os.path.join(dirpath, name)


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    added, skipped = [], []

    for path in find_rust_files(root):
        with open(path, "r", encoding="utf-8", newline="") as fh:
            text = fh.read()

        if MARKER in text:
            skipped.append(path)
            continue

        # Preserve a UTF-8 BOM if present, so the header lands after it and the
        # file's encoding is unchanged.
        bom = ""
        if text.startswith("\ufeff"):
            bom, text = "\ufeff", text[1:]

        # Normalise the first line ending so we do not create a mixed-ending file.
        with open(path, "w", encoding="utf-8", newline="") as fh:
            fh.write(bom + HEADER + text)
        added.append(path)

    print(f"header added : {len(added)}")
    print(f"already had  : {len(skipped)}")
    for p in added:
        print("   +", p)


if __name__ == "__main__":
    main()
