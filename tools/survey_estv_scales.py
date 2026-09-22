"""Survey the ESTV "Tarife" exports before importing them.

Reports, per canton per file: the splitting factor, the distinct Steuersubjekt
values, the number of bands, and the top band threshold. This matters because
cantons express marital treatment in different ways -- some use a splitting
factor, others publish separate scales per Steuersubjekt -- and merging those
rows would silently produce a wrong tariff.
"""

import glob
import importlib.util
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "xl", os.path.join(HERE, "xlsx_dump.py")
)
xl = importlib.util.module_from_spec(spec)
spec.loader.exec_module(xl)

# The ESTV "Tarife" exports live in `source-documents/` at the repository root, so
# the sweep finds them regardless of the working directory it is run from.
SOURCES = os.path.join(os.path.dirname(HERE), "source-documents")


def clean(t):
    if t is None:
        return ""
    return re.sub(r"\s+", " ", str(t).replace("\ufeff", "")).strip()


def to_float(t):
    t = clean(t)
    if not t:
        return None
    try:
        return float(t.replace("'", "").replace(",", "."))
    except ValueError:
        return None


files = sorted(glob.glob(os.path.join(SOURCES, "estv_scales_*.xlsx")))
print(f"found {len(files)} export files\n")

for path in files:
    sheets = dict(xl.read_xlsx(path))
    for sheet, rows in sheets.items():
        header_idx = None
        cols = {}
        for i, row in enumerate(rows):
            cells = [clean(c) for c in row]
            if "Kanton" in cells and "Steuerhoheit" in cells:
                header_idx = i
                cols = {n: j for j, n in enumerate(cells) if n}
                break
        if header_idx is None:
            print(f"{path}: no header found in sheet {sheet!r}")
            continue

        # Splitting factor lives above the header.
        splitting = None
        for row in rows[:header_idx]:
            cells = [clean(c) for c in row]
            for j, c in enumerate(cells):
                if c.lower().startswith("splittingfaktor"):
                    for later in cells[j + 1:]:
                        v = to_float(later)
                        if v is not None:
                            splitting = v
                            break

        # Header text varies between exports, so match on substrings. The code
        # column must NOT be confused with "Kantons-Id": matching "kanton" alone
        # finds the numeric id column first, which silently reports ids (19, 12,
        # 1) where canton codes are expected.
        def find_col(*needles, fallback=None, exclude=()):
            for j, name in enumerate([clean(c) for c in rows[header_idx]]):
                low = name.lower()
                if any(x.lower() in low for x in exclude):
                    continue
                if all(n.lower() in low for n in needles):
                    return j
            return fallback

        col_kanton = find_col("kanton", exclude=("id",), fallback=1)
        col_art = find_col("steuerart", fallback=2)
        col_subj = find_col("steuersubjekt", fallback=3)
        col_auth = find_col("steuerhoheit", fallback=4)
        col_width = find_col("chf", fallback=5)
        col_rate = find_col("zus", fallback=6)

        by_key = {}
        for row in rows[header_idx + 1:]:
            if len(row) <= col_rate:
                row = list(row) + [""] * (col_rate + 1 - len(row))
            code = clean(row[col_kanton])
            art = clean(row[col_art])
            subj = clean(row[col_subj])
            auth = clean(row[col_auth])
            width = to_float(row[col_width])
            rate = to_float(row[col_rate])
            if not code or width is None or rate is None:
                continue
            by_key.setdefault((code, art, subj, auth), []).append((width, rate))

        for (code, art, subj, auth), bands in sorted(by_key.items()):
            total = sum(w for w, _ in bands)
            top_rate = max(r for _, r in bands)
            print(
                f"{os.path.basename(path):22} {code:3} {art:10} split={splitting!s:6} "
                f"auth={auth:8} subj={subj[:34]:34} bands={len(bands):3} "
                f"top_threshold={total:10.0f} top_rate={top_rate:5.1f}%"
            )
    print()
