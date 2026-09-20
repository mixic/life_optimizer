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

| Canton | Cantonal Steuerfuss (2026) | Note |
|---|---|---|
| Zürich | 0.95 (95%) | Plus 119% from the city of Zürich |
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

#### Coverage: 22 of 26 cantons priceable

**Priceable (22):** AG AI AR BE BS GL GR JU LU NE NW OW SG SH SO SZ TG TI UR VD ZG ZH

Bern is priced by its standalone Stadt Bern table; the other 21 use the imported
scale (or flat rate, or BL's formulas) multiplied by the Steuerfuss.

**Refusing (4):** BL, FR, GE, VS — and all four now for the *same* reason: their
cantonal Steuerfuss is not a plain number in the ESTV workbook. That is a single
kind of missing input rather than four different pieces of engineering, which is
the substantive change the formula import made.

**Refusing (4):**

| Canton | Cause |
|---|---|
| `BL` | tariff imported, but its Steuerfuss cell is blank (see below) |
| `GE` | scale imported, but its Steuerfuss cell reads `148.5%9)` with a 12% rebate footnote |
| `VS` | scale imported, but its Steuerfuss cell is `3)` — *"Kein Vielfaches"* |
| `FR` | scale imported, but its Steuerfuss cell is blank |

#### Four tariff shapes

| Shape | Cantons | Representation |
|---|---|---|
| band **widths** (`Für die nächsten CHF`) | most | thresholds accumulated from widths |
| absolute **thresholds** (`Steuerbares Einkommen CHF` + `Grundbetrag CHF`) | Bund | thresholds taken directly |
| **flat percentage** (`Steuersatz %`) | OW, UR | `BaseScale::flat_rate_percent`, no bands |
| **formulas** (`Formel`) | BL | `BaseScale::formula_single` / `formula_married`, no bands |

Obwalden and Uri are the two cantons that levy a single uniform rate rather than
a progressive scale, and they need a separate multiplication rather than a band
walk. Their table is emitted as a single 0%-at-0 floor so the band machinery does
not double-count, with the percentage applied separately.

#### Basel-Landschaft: a tariff published as formulas

BL's export has no band column and no rate column. Each segment is an algebraic
expression in the taxable income, giving the **simple tax** directly:

```text
16731    -0.827548* $wert$ + 0.089722* $wert$ * (log $wert$ - 1) + 830.223746
44615    -0.328507* $wert$ + 0.043108 * $wert$ * (log $wert$ - 1) + (-1249.444454)
111538    0.051153* $wert$ + 0.010441 * $wert$ * (log $wert$ - 1) + (-4893.077017)
1282692   235687.5410 + 0.1862 * ($wert$ - 1282692)
```

Three decisions are worth recording, because each could silently produce a wrong
tax:

1. **The formulas are stored as source text, not transcribed coefficients.** The
   generator emits the expression into `estv_scales_data.rs` and
   `federal_tax::eval_formula` evaluates it, so there is no transcription step in
   which a digit can be lost. The evaluator understands `+ - * /`, parentheses,
   decimals, `$wert$`, and `log` — **the natural logarithm**, applied as a prefix
   operator: `log $wert$`, with no parentheses. An earlier attempt required
   `log(`, which refused every real expression.
2. **Segments are alternatives, not increments.** The last segment whose threshold
   does not exceed the income supplies the *whole* tax. That is what makes the
   expressions self-consistent: evaluated at each other's thresholds they agree to
   under 0.001 CHF, which `basel_landschaft_segments_are_continuous_at_their_thresholds`
   asserts. A progressive tariff cannot jump when a band changes, so that
   continuity is a check on the *source data*.
3. **Splitting is not applied.** BL publishes separate expressions per marital
   status, so the marital difference is already in the tariff. The married table
   adds one segment the single table does not have: a `0.49 * $wert$ / 100` relief
   band from CHF 8,366.

The tariff computes correctly today — but **BL is still not priceable**, because
the Liestal row of the ESTV Steuerfuss workbook is *empty* for every year the
workbook covers (2024, 2025 and 2026 alike). So BL is blocked on its multiplier,
not its tariff, and `canton_tax_data(BL).missing_fields()` reports exactly those
two multipliers. Supplying a sourced Steuerfuss is now the only thing between BL
and a figure; nothing in the code needs to change.

#### Steuerfuss vintage — resolved

Scales come from **2026** exports, and the Steuerfüsse are now read from the
**2026** row of the ESTV workbook, so the two halves of the product describe the
same tax regime. `cantons::SELF_ASSESSMENT_YEAR` holds the year, and every
workbook-filled multiplier carries it in its `Provenance`; a mismatch would fail
`steuerfuss_vintage_matches_the_scale_vintage`.

This corrected three cantons whose multipliers had moved between the years:

| Canton | 2024 | 2026 |
|---|---|---|
| Aargau | 1.12 | 1.03 |
| Obwalden (Sarnen) | 3.35 | 3.25 |
| Zürich | 0.98 | 0.95 |

Aargau and Zürich were previously *hand-entered* in the registry at 1.11 and
0.98 — values that matched neither year's workbook cell exactly (Aargau's 2024
cell says 1.12). Those overrides are gone: the workbook is now the single source,
so a figure cannot drift away from the file it claims to come from.

### Cantonal simple-tax scales — imported

Scales are imported from ESTV "Tarife" exports by
`tools/import_estv_scales.py`, which reads **every canton present in a file**, so
one workbook can add several. Current coverage:

Priceable end to end today: the 22 cantons listed above.

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

| | Assessable | Simple tax | x Steuerfuss 1.99 | Effective |
|---|---|---|---|---|
| Single | 100,000 | 6,938 | 13,806.62 | 13.81% |
| Married | 50,000 (split) | 2,384 | 9,488.32 | 9.49% |

The married figure is **not** half the single one (6,903.31). The scale is
progressive, so two halves at 50,000 cost less than one whole at 100,000 — that
difference is exactly what income splitting grants, and it is why the split
cannot be approximated by halving the result.

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

### Federal tariff — **VERIFIED against the statute**

The federal tariff is imported from the official ESTV export
`estv_scales_Bund.xlsx` and its values have been reconciled against the
statutory text of **DBG Art. 36** as published in `SR 642.11.pdf`.

The statute reads (abridged):

```text
bis 15 200 Franken Einkommen   0.00 und fuer je weitere 100 Franken 0.77
fuer 33 200 Franken Einkommen 138.60 und fuer je weitere 100 Franken 0.88 mehr
fuer 43 500 Franken Einkommen 229.20 und fuer je weitere 100 Franken 2.64 mehr
fuer 58 000 Franken Einkommen 612.00 und fuer je weitere 100 Franken 2.97 mehr
```

Every threshold, marginal rate and base amount agrees with the imported table.
`federal_tax::tests::statute_values_match_the_imported_grid` asserts the
thresholds, rates and computed base amounts, so a regeneration cannot silently
break the reconciliation.

**Two errors this corrected**, both worth recording because they were
introduced here rather than found in the data:

1. The original hand-entered federal table had a top marginal rate of **7.39%**
   against the statutory maximum of **13.2%**, plus a non-monotonic segment.
2. A hand computation quoted during development claimed an average federal
   burden of about **1.80%** at CHF 100,000 taxable. That was wrong — it used
   misremembered bands. The correct figure under this tariff is **~2.69%**, and
   the statute confirms the bands producing it.

#### A trap in the federal export's format

The federal export uses a **different column layout** from the cantonal ones,
and conflating them silently changes the tariff:

| Export | Band column | Meaning | Extra column |
|---|---|---|---|
| Cantonal | `Für die nächsten CHF` | band **width** | — |
| Federal | `Steuerbares Einkommen CHF` | absolute **threshold** | `Grundbetrag CHF` base amount |

Treating the federal thresholds as widths produces a wholly different schedule.
Note also that several headers contain "CHF", so a naive substring match on `chf`
finds `Grundbetrag CHF` instead of `Für die nächsten CHF` — which is exactly the
bug that briefly corrupted the cantonal scales during this work.

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
