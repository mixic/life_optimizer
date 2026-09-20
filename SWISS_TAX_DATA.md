# Swiss Tax Data: Source, Status, and What Is Still Needed

This document records exactly where every Swiss tax number in this repository
comes from, and — more importantly — where the gaps are. It exists because
`FutureWork.md` §7 sets the standard this project holds itself to: numbers must
be traceable "to a data source, to a named assumption, to a documented
estimation method — versus being a plausible narrative wrapped around unfitted
parameters."

A tax engine is the worst place to guess. Cantonal tax varies by roughly a
factor of three across Switzerland, so a wrong multiplier changes every
recommendation the tool makes while still looking authoritative.

---

## 1. The two-level model

Swiss income tax is the sum of two separately computed layers:

```
total tax = federal(income)
          + cantonal_base(income) × cantonal_Steuerfuss / 100
          + cantonal_base(income) × municipal_Steuerfuss / 100
```

- **Federal direct tax** is progressive and **identical in all 26 cantons**. It
  differs only by marital status. It is its own tariff, *not* a scaled cantonal
  base.
- **Cantonal tax** is the canton's *simple tax* ("einfache Steuer") multiplied
  by a **Steuerfuss** — a percentage applied uniformly to the whole scale.
- **Municipal tax** uses the same simple tax as the cantonal layer, multiplied
  by the municipality's own Steuerfuss.

The Steuerfuss design is what makes a canton's tax level summarisable by a
single number once its base scale is known. It is also why the base scale and
the multiplier have to come from different sources.

---

## 2. What is sourced today

### Steuerfüsse — **sourced and generated**

The two workbooks at the repository root are official publications of the
Eidgenössische Steuerverwaltung (ESTV):

| File | Content | Relevant? |
|---|---|---|
| `steuerfuesse-np-1995-2026.xlsx` | *"Steuerfüsse in den Kantonshauptorten"* — income and wealth taxes of **natural persons** | **Yes** — this is the one that matters |
| `steuerfuesse-jp-1995-2026.xlsx` | Profit and capital taxes of **legal persons** (companies) | No — corporate tax does not apply to personal work-life planning |

`tools/generate_steuerfuss.py` reads the natural-persons workbook and emits
`src/canton_steuerfuss_data.rs`, containing **832 rows — 26 cantons × 32 years
(1995–2026)** — each carrying:

- `cantonal` / `municipal` multipliers, where the source expressed them as plain
  numbers
- `cantonal_raw` / `municipal_raw`, the source cell text verbatim
- `cantonal_blank` / `municipal_blank`, distinguishing "the multiplier does not
  apply" from "we could not read this cell"

Regenerate with:

```bash
python tools/generate_steuerfuss.py steuerfuesse-np-1995-2026.xlsx src/canton_steuerfuss_data.rs
```

### Why the raw text and the blank flag exist

The ESTV sheets do not express every cell as a plain multiplier. Real examples
the generator preserves rather than "fixes":

| Canton | Cell | Footnote meaning |
|---|---|---|
| Genève | `147.5%9)` | A 12% rebate applies: *"Rabais de 12% de l'impôt cantonal de 147,5%"* |
| Basel-Stadt | municipal `4)` | *"Die Gemeindesteuer ist in der Kantonssteuer inbegriffen"* — municipal tax is already inside the cantonal figure |
| Chur (GR) | municipal `88% 7)` | A percentage **of the cantonal tax**, not a multiple of the simple tax |
| Sion (VS) | cantonal `3)` | *"Kein Vielfaches"* — no multiplier at all |
| Ticino, Vaud | church tax `-` | Genuinely not applicable — this is the `blank` case |

Treating any of those as a number would be wrong in a different way each time.
So the generator emits `None` plus the original text, and
`tests/steuerfuss_data.rs::documented_source_exceptions_are_preserved` fails if
a future regeneration silently turns one of them into a value.

### Notable anchors

| Canton | Cantonal Steuerfuss (2024) | Note |
|---|---|---|
| Zürich | 0.98 (98%) | Plus 119% from the city of Zürich |
| Bern | 2.975 (297.5%) | A high-tax canton — nearly three times the simple tax |
| Zug | 0.78 (78%) | Among the lowest |

The spread between Zug and Bern is the substantive fact that makes "assume your
canton is like Zürich" indefensible.

---

## 3. What is still missing

### Published cross-check figures — **available, and they expose a real gap**

The only externally-sourced cantonal figures in this repository come from
finpension.ch, *"Steuerprogression in der Schweiz"* (16.06.2026), which quotes
the ESTV Steuerrechner for **Steuerjahr 2025** under the explicit assumption
"alleinstehend, konfessionslos, keine Kinder". Total tax (Bund + Kanton +
Gemeinde), in CHF, against **taxable** income:

| Taxable income | Schwyz | Zürich | Bern |
|---|---|---|---|
| CHF 25,000 | 1,643 | 1,364 | 4,002 |
| CHF 50,000 | 4,424 | 4,854 | 9,250 |
| CHF 100,000 | 12,327 | 16,181 | 22,984 |
| CHF 250,000 | 46,732 | 68,897 | 80,382 |
| CHF 500,000 | 113,200 | 172,133 | 185,468 |

`tests/published_tax_reference.rs` pins these figures and the cantonal ordering
they imply. Two things worth noting:

- **The ordering is income-dependent.** Schwyz is *not* the cheapest at
  CHF 25,000 — Zürich is (1,364 vs 1,643). Schwyz becomes cheapest from
  CHF 50,000 up. Bern is dearest at every level. A model that assumed a fixed
  cantonal ranking would be wrong at the low end.
- **Bern's shipped table under-taxes against these figures**, by 5.7 to 12.4
  percentage points:

  | Taxable income | Model (Bern table) | Published | Difference |
  |---|---|---|---|
  | CHF 25,000 | 5.41% | 16.01% | −10.60pp |
  | CHF 50,000 | 12.00% | 18.50% | −6.50pp |
  | CHF 100,000 | 17.24% | 22.98% | −5.74pp |
  | CHF 250,000 | 24.73% | 32.15% | −7.42pp |
  | CHF 500,000 | 24.73% | 37.09% | −12.36pp |

  Two causes are visible in the numbers. First, `TaxSchedule::bern_city_default`
  is Steuerjahr 2024 while these figures are 2025. Second, and more
  significantly, that table has **no entries above CHF 200,000**, so
  `lookup_tax_rate` clamps to its last value — which is why the model returns an
  identical 24.73% at both CHF 250,000 and CHF 500,000, while the published
  burden keeps climbing. The published figures also include the federal share,
  which the table does not itemise.

  This does **not** invalidate the table for the mid-range incomes the tool is
  mostly used at (the smallest gap, −5.74pp, is at CHF 100,000), but it means
  Bern output above roughly CHF 200,000 taxable should not be trusted, and the
  whole table should be replaced when a proper Bern tariff is available.

---

### Cantonal simple-tax scales — **8 of 26 imported**

Scales are imported from ESTV "Tarife" exports by
`tools/import_estv_scales.py`, which reads **every canton present in a file**, so
one workbook can add several. Current coverage:

| Canton | Treatment | Top rate |
|---|---|---|
| `AG` | shared scale, splitting 2.0 | 11.0% |
| `BS` | per-subject scales (married = 2x single) | 28.2% |
| `LU` | per-subject scales | 5.8% |
| `ZH` | per-subject scales | 13.0% |
| `SH` | shared scale, splitting 1.9 | 12.0% |
| `SO` | shared scale, splitting 1.9 | 11.5% |

Priceable end to end today: **AG, BS, LU, SH, SO, ZH** (six), plus **Bern** via
its legacy standalone table.

Imported but **not** priceable, because the canton has no usable multiplier:

| Canton | Why |
|---|---|
| `GE` | scale imported, but its Steuerfuss cell reads `148.5%9)` with footnote 9: a 12% rebate applies |
| `FR` | scale imported, but its Steuerfuss cell is blank |

The remaining 18 cantons have neither a scale nor (for some) a multiplier, and
refuse loudly.

#### Marital treatment is expressed in two incompatible ways

This is the subtlety most likely to cause a wrong answer, so it is worth stating
plainly. Cantons do one of:

* publish **one shared scale** (`Steuersubjekt = Alle`) and express the marital
  difference through a **splitting factor** — AG (2.0), SH (1.9), SO (1.9); or
* publish **separate scales per `Steuersubjekt`**, where the difference is
  already in the table and splitting is **not** applied — BS, LU, ZH.

Applying splitting on top of a married-specific scale would count the marital
adjustment twice. `BaseScale` therefore carries both axes and
`BaseScale::splitting_factor(married)` returns a value only for a married
taxpayer on a shared scale. The imported data records which shape each canton
uses rather than inferring it.

#### Verified figures

Aargau at CHF 100,000 taxable, hand-checked against the ESTV bands:

| | Assessable | Simple tax | x Steuerfuss 2.07 | Effective |
|---|---|---|---|---|
| Single | 100,000 | 6,938 | 14,361.66 | 14.36% |
| Married | 50,000 (split) | 2,488 | 9,869.76 | 9.87% |

Four cantons are **documented source exceptions** with no plain multiplier,
asserted exactly by `source_exception_set_is_exactly_as_documented`:

| Canton | Source cell | Why no multiplier |
|---|---|---|
| `GE` | `148.5%9)` | Footnote 9: a 12% rebate applies, so the printed figure is not the effective multiplier |
| `VS` | `3)` | Footnote 3: *"Kein Vielfaches"* — Valais expresses no cantonal multiplier |
| `BL` | blank | Nothing to read |
| `FR` | blank | Fribourg uses separate income and wealth rows |

So the outstanding data is:

1. **`base_scale` for 18 cantons** — from an ESTV "Tarife" export, the same
   `.xlsx` shape as the eight already imported.
2. **An effective multiplier for `GE`, `VS`, `BL`, `FR`** — see the table above.
   For GE the arithmetic is determinate (147.5% x 0.88 = 129.8%) but
   interpreting a rebate footnote is a decision, not a parse, so it is
   deliberately left unsupplied.
3. **The federal tariff values** — see the next section.

Each canton requires:

1. **`steuerfuss_percent`** — available from the ESTV workbook above (done)
2. **`base_scale`** — the canton's simple-tax tariff: a list of
   `{ threshold, rate }` brackets. Published in the canton's Steuer- or
   Steuergesetz, and tabulated in ESTV's income-tax tariff publications.
3. **`capital_municipal_fuss_percent`** — available from the workbook (done)

So item 2 is the outstanding piece: **26 bracket tables**, plus the marital
treatment for each (a married scale, a splitting procedure, or a deduction —
cantons differ).

To supply them, the format needed per canton is:

```
canton,threshold,rate      # rate as a decimal fraction of taxable income
ZH,0,0.00
ZH,4700,0.02
...
```

### Federal tariff — **NOT PRESENT**

**Correction.** The file `Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf` in this
repository is **canton Bern's tariff, not a federal one**: `Art. 42 Abs. 1` is
Bern's own withholding article, so it does not provide a schedule shared by all
26 cantons. There is currently **no federal tariff source in this repository**.

`src/federal_tax.rs` still contains a bracket table marked
`FEDERAL_TARIFF_IS_VERIFIED = false`, with a **known defect**: the marginal rate
decreases across the 72,500 → 78,100 → 103,600 segment, which no progressive
tariff can do. Those figures appear to be ESTV "ans Satz" values — the *average*
rate at that threshold — rather than marginal rates.

It was not deleted because the two-level structure and its tests need something
to exercise, but:

- `monotonicity_violations()` reports exactly which brackets are malformed
- `tariff_flagged_verified_must_be_monotonic` fails if the flag is raised
  without fixing the data
- nothing that acts on a projection should trust it until the flag is `true`

**Needed:** the correct federal tariff, with its source named. Note that the
published schedule may be expressed as a **"Einheits-Satz"** (a uniform rate
applied to the whole taxable income) rather than marginal brackets — the federal
withholding tables use that form, so the `FederalBracket` model may need a
sibling representation.

### The Bern tariff PDF — decoded partially, and why it stops there

`tools/pdf_text.py` and `tools/characterise_bern_tokens.py` record what is
recoverable from that PDF, so nobody repeats this work:

- It is **vector text, not scanned images** — the file contains no raster images
  at all, only `FlateDecode` streams.
- Its text uses **Identity-H** encoding, so `TJ` operands are glyph indices. One
  CMap is visible, covering exactly 12 glyphs: digits `0-9`, `.`, and a
  thousands separator mapped to U+2019.
- With that CMap, **all 244,498 glyph codes resolve** and **21,979 numeric
  tokens** are recovered — income levels (to CHF 6.2bn) and 10,828 tax amounts
  in `1'002.95` form.
- **The rate column ("Einheits-Satz [%]") is not recoverable.** Rates have the
  shape `1.5500` (one digit, four decimals) and the decode yields **zero**
  4-decimal tokens: that column's decimal-point glyph is absent from the visible
  CMap, so rates fragment into a leading digit and a trailing integer. The
  remaining CMaps sit inside compressed object streams, which requires a full
  PDF object parser (xref / object-stream traversal), not stdlib `zlib`.

Two parsing traps, both hit in practice and both worth knowing:

1. The thousands separator is **U+2019 (RIGHT SINGLE QUOTATION MARK)**, not
   ASCII `'`. Treating only ASCII as a separator splits `1'000` into `1` and
   `000`.
2. A token regex such as `\d+(?:\.\d+)?` **silently discards** every value
   containing a separator — which, in a thousands-separated document, is nearly
   all of them. This is why the rate column first looked *missing* rather than
   *unparsed*.

**What this is worth, and what it is not.** A fully decoded Bern table would
give a second, independent check on the hand-entered
`TaxSchedule::bern_city_default` figures. It would **not** unblock the other 25
cantons, which is the actual requirement, so it is off the critical path.

---

## 4. What `--canton` does today

`--canton` is wired to the tax data, and behaves in one of three explicit ways:

| Invocation | Behaviour |
|---|---|
| `--canton` omitted | Uses **Bern**, the one canton with a complete table, and prints that it did so |
| `--canton BE` | Uses Bern's standalone Stadt Bern table |
| `--canton XX` (unknown code) | Exits 2, listing the valid codes |
| `--canton ZH` (known, no scale loaded) | **Exits 2** with what is missing and two ways forward |
| `--custom-tax-rate 0.14` | Bypasses canton resolution entirely |

This replaced a genuinely misleading behaviour: `--canton` was accepted,
defaulted to `"ZH"`, and **silently ignored** — so `--canton ZH` produced Bern
numbers. A flag that appears to change the answer but does not is worse than an
absent flag, because the user has no way to notice.

The failure message for an unloaded canton is deliberately specific:

```
error: cannot price canton ZH (Zürich). Missing: cantonal base tax scale.

  Cantonal tax = simple_tax(income) x Steuerfuss. The Steuerfuss is loaded from
  the ESTV workbook, but the ZH cantonal simple-tax scale has not been supplied yet.

  Two ways forward:
    - omit --canton to use Bern, the one canton with a complete table
    - pass --custom-tax-rate with your observed rate from your tax assessment (most accurate)

  See SWISS_TAX_DATA.md §3 for the table format needed.
```

`tests/cli_canton.rs` pins all five rows of that table, including that no
projection is printed alongside the error.

---

## 5. What works in the meantime

There are two honest paths today:

1. **`--custom-tax-rate`** — pass your own observed effective rate as a decimal
   (e.g. `0.1382` for 13.82%), taken from your tax assessment. This bypasses the
   canton tables entirely and is the most accurate option available, because it
   uses your actual figure rather than a modelled one.
2. **Bern** — `TaxSchedule::bern_city_default` carries the original hand-entered
   Stadt Bern table (cantonal + municipal + church combined), sourced from the
   city's own `Steuerbelastung des Arbeitseinkommens` publication.

---

## 6. Current status summary

| Piece | Status | Location |
|---|---|---|
| 26 cantons with codes, names, capitals | Shipped | `src/cantons.rs` |
| Steuerfüsse, 26 cantons × 32 years | **Shipped, sourced to ESTV** | `src/canton_steuerfuss_data.rs` |
| Source-exception preservation (rebates, footnotes, blanks) | Shipped + tested | generator, `tests/steuerfuss_data.rs` |
| Federal bracket structure + tariff arithmetic | Shipped | `src/federal_tax.rs` |
| Federal bracket **values** | **Unverified — known defect** | `src/federal_tax.rs` |
| Cantonal simple-tax scales (26 tables) | **Not sourced — blocks cantons** | needed |
| Marital treatment per canton | **Not sourced** | needed |
| `--canton` wired, fails loudly on unloaded cantons | **Shipped + tested** | `src/main.rs`, `tests/cli_canton.rs` |

**What unblocks the rest:** the 26 cantonal simple-tax scales (§3), and the
corrected federal bracket table (§3). Both are data, not code — the calculation
path, the provenance model, and the failure behaviour are already in place and
tested, so loading either is a data-entry step rather than new engineering.
