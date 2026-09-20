"""Diagnose ESTV exports the survey parser drops.

For each file, print the canton codes found, the sheet name, and the header row,
so a file that yields no rows reveals why (different header, different sheet
layout, or an empty export).
"""

import glob
import importlib.util
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("xl", os.path.join(HERE, "xlsx_dump.py"))
xl = importlib.util.module_from_spec(spec)
spec.loader.exec_module(xl)


def clean(t):
    return re.sub(r"\s+", " ", str(t or "").replace("\ufeff", "")).strip()


expected = set("AG AI AR BE BL BS FR GE GL GR JU LU NE NW OW SG SH SO SZ TG TI UR VD VS ZG ZH".split())
seen = set()

print(f"{'file':26} {'sheets':8} {'header row':60} cantons")
for path in sorted(glob.glob("estv_scales_*.xlsx")):
    name = os.path.basename(path)
    sheets = list(xl.read_xlsx(path))
    sheet_names = [n for n, _ in sheets]
    header = ""
    codes = []
    for sheet, rows in sheets:
        for row in rows:
            cells = [clean(c) for c in row]
            if "Kanton" in cells and "Steuerhoheit" in cells:
                header = " | ".join(cells[:7])
                ki = cells.index("Kanton")
                for r in rows[rows.index(row) + 1:]:
                    if len(r) > ki:
                        c = clean(r[ki]).upper()
                        if len(c) == 2 and c.isalpha() and c not in codes:
                            codes.append(c)
                break
    seen.update(codes)
    flag = "" if codes else "   <-- NO CANTON FOUND"
    print(f"{name:26} {len(sheet_names):<8} {header[:58]:60} {','.join(codes)}{flag}")

missing = sorted(expected - seen)
print()
print(f"cantons never seen: {missing if missing else 'none'}")
