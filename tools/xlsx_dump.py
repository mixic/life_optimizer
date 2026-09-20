"""Minimal read-only .xlsx inspector using only the Python standard library.

An .xlsx file is a ZIP archive of XML parts. This reads the workbook metadata to
map sheet names to sheet files, the shared-strings table, and then each sheet's
cells -- enough to dump the data without third-party dependencies.
"""

import sys
import zipfile
import re
import xml.etree.ElementTree as ET

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
REL_NS = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"


def col_to_index(cell_ref):
    """Convert a cell reference like 'BC12' to a zero-based column index."""
    letters = re.match(r"[A-Z]+", cell_ref).group(0)
    index = 0
    for ch in letters:
        index = index * 26 + (ord(ch) - ord("A") + 1)
    return index - 1


def read_xlsx(path):
    """Yield (sheet_name, rows) where rows is a list of lists of strings."""
    with zipfile.ZipFile(path) as z:
        # Shared strings: most text cells reference this table by index.
        shared = []
        if "xl/sharedStrings.xml" in z.namelist():
            root = ET.fromstring(z.read("xl/sharedStrings.xml"))
            for si in root.findall(f"{NS}si"):
                # A string can be split across multiple <t> runs.
                text = "".join(t.text or "" for t in si.iter(f"{NS}t"))
                shared.append(text)

        # Map sheet names to their part names via the workbook relationships.
        workbook = ET.fromstring(z.read("xl/workbook.xml"))
        rels = ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))
        rel_targets = {
            rel.get("Id"): rel.get("Target") for rel in rels
        }

        sheets = []
        for sheet in workbook.find(f"{NS}sheets"):
            name = sheet.get("name")
            rel_id = sheet.get(f"{REL_NS}id")
            target = rel_targets.get(rel_id, "")
            if not target.startswith("xl/"):
                target = "xl/" + target.lstrip("/")
            sheets.append((name, target))

        for name, target in sheets:
            if target not in z.namelist():
                continue
            root = ET.fromstring(z.read(target))
            rows = []
            for row in root.iter(f"{NS}row"):
                cells = {}
                for c in row.findall(f"{NS}c"):
                    ref = c.get("r") or ""
                    ctype = c.get("t")
                    value_el = c.find(f"{NS}v")
                    if ctype == "inlineStr":
                        text = "".join(
                            t.text or "" for t in c.iter(f"{NS}t")
                        )
                    elif value_el is None:
                        text = ""
                    elif ctype == "s":
                        idx = int(value_el.text)
                        text = shared[idx] if 0 <= idx < len(shared) else ""
                    else:
                        text = value_el.text or ""
                    cells[col_to_index(ref)] = text
                if cells:
                    width = max(cells) + 1
                    rows.append([cells.get(i, "") for i in range(width)])
                else:
                    rows.append([])
            yield name, rows


def main():
    path = sys.argv[1]
    max_rows = int(sys.argv[2]) if len(sys.argv) > 2 else 25
    for name, rows in read_xlsx(path):
        print(f"===== SHEET: {name}  ({len(rows)} rows) =====")
        for i, row in enumerate(rows[:max_rows]):
            trimmed = [c for c in row]
            while trimmed and trimmed[-1] == "":
                trimmed.pop()
            print(f"{i:4d} | " + " | ".join(trimmed))
        if len(rows) > max_rows:
            print(f"     ... {len(rows) - max_rows} more rows")
        print()


if __name__ == "__main__":
    main()
