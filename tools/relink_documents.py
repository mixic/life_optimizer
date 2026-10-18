#!/usr/bin/env python3
"""Rewrite and then verify every cross-reference after moving the essays into documents/.

Copyright (C) 2026 MILAN NIKOLIC
SPDX-License-Identifier: GPL-3.0-or-later

Usage:
    python tools/relink_documents.py            # rewrite, then verify
    python tools/relink_documents.py --check    # verify only

# Why a script

Moving twenty-three files changes the relative path of every link between them and the
tool-facing documents that stayed at the root. Doing that by hand and *claiming* it worked is
the failure mode: a broken link looks exactly like a working one until someone clicks it. So
the rewrite is mechanical and the verification is exhaustive -- every markdown link in every
document is resolved against the filesystem, and the script exits non-zero listing any that do
not land on a file.

# What it does not touch

External URLs, anchors, and links to files outside the markdown set. Code blocks are skipped
too, because a path shown in a shell command is not a link.
"""

from __future__ import annotations

import os
import re
import sys

ROOT = "."
DOCS = "documents"

# Every markdown session link: [text](target) or [text](target#anchor)
LINK = re.compile(r"\]\(([^)\s]+)\)")


def markdown_files() -> list[str]:
    files = []
    for name in sorted(os.listdir(ROOT)):
        if name.lower().endswith(".md") and os.path.isfile(os.path.join(ROOT, name)):
            files.append(name)
    if os.path.isdir(DOCS):
        for name in sorted(os.listdir(DOCS)):
            if name.lower().endswith(".md"):
                files.append(os.path.join(DOCS, name))
    return files


def is_local_markdown(target: str) -> bool:
    if target.startswith(("http://", "https://", "mailto:", "#")):
        return False
    path = target.split("#", 1)[0]
    return path.lower().endswith(".md")


def resolve(source: str, target: str) -> str | None:
    """The filesystem path a link from `source` to `target` should land on."""
    path = target.split("#", 1)[0]
    if not path:
        return None
    base = os.path.dirname(source)
    candidate = os.path.normpath(os.path.join(base, path))
    return candidate if os.path.isfile(candidate) else None


def rewrite_link(source: str, target: str) -> str:
    """Return the target as it should be written from `source`'s new location.

    Two rules, and they are inverses:
      - a document in documents/ linking to a root-kept file needs `../`
      - a root file linking into documents/ needs `documents/`
    A link that already resolves is left exactly as it was, so re-running is safe.
    """
    if resolve(source, target) is not None:
        return target
    path, _, anchor = target.partition("#")
    suffix = f"#{anchor}" if anchor else ""
    name = os.path.basename(path)
    in_docs = os.path.dirname(source) == DOCS
    if in_docs:
        # Try the parent, which is where root-kept files live.
        candidate = f"../{path.lstrip('./')}"
        if resolve(source, candidate + suffix) is not None:
            return candidate + suffix
        # Or a sibling within documents/.
        candidate = name
        if resolve(source, candidate + suffix) is not None:
            return candidate + suffix
    else:
        candidate = f"{DOCS}/{path.lstrip('./')}"
        if resolve(source, candidate + suffix) is not None:
            return candidate + suffix
    return target


def strip_code(text: str) -> list[tuple[int, str]]:
    """Yield (line number, line) for lines outside fenced code blocks."""
    inside = False
    out = []
    for index, line in enumerate(text.splitlines()):
        if line.strip().startswith("```"):
            inside = not inside
            continue
        out.append((index, line) if not inside else (index, None))
    return out


def rewrite() -> int:
    changed = 0
    for source in markdown_files():
        with open(source, encoding="utf-8") as handle:
            text = handle.read()
        lines = text.split("\n")
        inside = False
        edits = 0
        for index, line in enumerate(lines):
            if line.strip().startswith("```"):
                inside = not inside
                continue
            if inside:
                continue

            def replace(match: re.Match[str]) -> str:
                nonlocal edits
                target = match.group(1)
                if not is_local_markdown(target):
                    return match.group(0)
                new = rewrite_link(source, target)
                if new != target:
                    edits += 1
                return f"]({new})"

            lines[index] = LINK.sub(replace, line)
        if edits:
            with open(source, "w", encoding="utf-8", newline="\n") as handle:
                handle.write("\n".join(lines))
            print(f"rewrote {edits:>3} link(s) in {source}")
            changed += edits
    print(f"\n{changed} link(s) rewritten")
    return changed


def verify() -> int:
    broken = []
    checked = 0
    for source in markdown_files():
        with open(source, encoding="utf-8") as handle:
            text = handle.read()
        inside = False
        for number, line in enumerate(text.split("\n"), start=1):
            if line.strip().startswith("```"):
                inside = not inside
                continue
            if inside:
                continue
            for match in LINK.finditer(line):
                target = match.group(1)
                if not is_local_markdown(target):
                    continue
                checked += 1
                if resolve(source, target) is None:
                    broken.append(f"{source}:{number} -> {target}")
    print(f"checked {checked} local markdown link(s)")
    if broken:
        print(f"\nBROKEN ({len(broken)}):")
        for entry in broken:
            print(f"  {entry}")
        return 1
    print("every local markdown link resolves")
    return 0


def main() -> int:
    check_only = "--check" in sys.argv
    if not check_only:
        rewrite()
    return verify()


if __name__ == "__main__":
    sys.exit(main())
