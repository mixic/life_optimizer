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

  These five figures are pinned to a band by `bern_table_is_close_to_the_published_figures`,
  which is worth more than the loose tolerance it replaced. The model figures were
  briefly *different* — 5.41% became 14.70%, 12.00% became 10.56% — because of the
  rate-base defect described below. Restoring them to the documented table is what
  confirmed the fix rather than merely asserting it.

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

#### Coverage: 23 of 26 cantons priceable

**Priceable (23):** AG AI AR BE BL BS GL GR JU LU NE NW OW SG SH SO SZ TG TI UR VD ZG ZH

Bern is priced by its standalone Stadt Bern table; the other 22 use the imported
scale (or flat rate, or BL's formulas) multiplied by the Steuerfuss.

**Refusing (3):**

| Canton | Cause |
|---|---|
| `GE` | scale imported, but its Steuerfuss cell reads `148.5%9)` with a 12% rebate footnote |
| `VS` | scale imported, but its Steuerfuss cell is `3)` — *"Kein Vielfaches"* |
| `FR` | scale imported, but its Steuerfuss cell is blank |

All three refuse for the *same* reason: their cantonal Steuerfuss is not a plain
number in the ESTV workbook. `BL` used to be on this list and left it when its
legally-fixed 100% multiplier was sourced — not when its tariff was implemented,
which had happened earlier and changed nothing.

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

The tariff computes correctly — and **BL is now priceable**, since its multiplier
turned out to be determinate even though the workbook is blank.

The ESTV Steuerfuss workbook leaves the Liestal row **entirely empty for all 32
years it covers** — both the cantonal and the municipal cell. The reason is
structural rather than a gap: BL does not adjust its multiplier annually. The
**Steuerfussdekret (SGS 331.2)** fixes the cantonal Steuerfuss at **100% of the
normal state tax**, and when revenue has to change, the canton amends the *tariff
brackets* instead. That fits BL being the only canton here whose tariff is
published as formulas rather than a band table.

The 100% figure is stated verbatim in a Landrat *Vorlage* restating the decree:
*"Der kantonale Einkommenssteuerfuss für das Steuerjahr … beträgt 100 Prozent der
normalen Staatsteuer vom Einkommen der natürlichen Personen"*. Liestal's municipal
multiplier is **65%**.

Source strength differs between the two figures and is recorded in the code as
such: the cantonal 100% comes from the decree itself (SGS 331.2, in force
01.01.2022), while the municipal 65% comes from a cantonal tax comparison rather
than an ESTV publication — the weakest figure in the entry, and labelled that way
so a better source can replace it without touching anything else.

BL at CHF 100,000 taxable prices at **13.5%** effective (simple tax × 1.65).

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

Priceable end to end today: the 23 cantons listed above.

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

## 4. Deductions — the sourced rule data

`src/estv_deductions_data.rs` is generated by
`tools/import_estv_deductions.py` from two further ESTV exports:

| File | Rows | What it holds |
|---|---|---|
| `estv_deductions.xlsx` | 618 → 617 merged | per canton per deduction: `Betrag`, `Prozent`, `Minimum`, `Maximum` |
| `estv_deduction_scales.xlsx` | 466 → 22 scales | phase-out tables: `Reineinkommen` → `Abzug CHF` |

Both are tax year **2025**. Coverage is complete and even: 27 jurisdictions
(26 cantons + `Bund`) × the same deduction categories, so a canton missing a rule
is visible as a gap rather than a silent default.

### The rule shape is a combination, not a choice

```text
deduction = clamp(amount + percent/100 x base, minimum, maximum)
```

`amount` and `percent` are **not** alternatives. Aargau's
`Pauschalabzug übrige Berufskosten` is `Betrag=700, Prozent=10, Maximum=2400` —
"700 plus 10%, capped at 2400". A model that picks one column cannot express it.
`Maximum = 0` means the source states *no ceiling*, which is not a ceiling of
zero; treating it as one would zero out most of the table.

### The rules file also contains non-deductions

This is the trap the data module exists to close. The same flat table mixes:

| Kind | Rows | Example | Why it must not be summed |
|---|---|---|---|
| `Deduction` | 574 | `Kinderabzug` | the real thing |
| `Threshold` | 9 | `Schwellwert für Abzug AHV-/IV-Rentner` | a *breakpoint* used by another rule |
| `Other` | 34 | `Steuerermässigung pro Kind` | a tax relief, not a deduction of income |
| `Other` | | `Maximaler technischer Zinssatz Einmaleinlage in CHF FINMA` | a rate constant (350 = 3.5%) stored in the same `Prozent` column |

Summing every row would both double-count and treat a threshold as an amount. So
each row carries its classification, the generator prints anything it could not
classify, and the tests assert the classification is *used* rather than merely
present — including that the `Prozent <= 100` sanity bound applies to deductions
only, since the FINMA rate constant is legitimately 350.

### Rows that are parts of one rule

Valais's `Pauschalabzug Unterhaltskosten von vermieteten Liegenschaften mit Alter
bis 20 Jahren` appears **twice**: once as `Prozent=10, Maximum=0` and once as
`Prozent=0, Maximum=15000`. That is one rule, not two, and keeping the rows
separate would mean a consumer finds whichever it reads first and silently drops
either the rate or the cap. The importer merges by `(canton, name)`, taking the
non-zero value of each field, and reports a **conflict** if two rows disagree on a
field — which would mean the source itself is inconsistent, not that row order
should decide.

### Phase-out scales, and the circularity that dissolved

The means-tested deductions are step functions of net income that phase *down* to
zero, so `DeductionScale::amount_at` walks the steps rather than interpolating:

```text
FR  Abzug für bescheidenes Einkommen, Ledige ohne Kind
    4100 at 0, 3900 at 20301, 3700 at 21301, ... 100 at 39301, 0 at 40301
```

Every scale in the export phases to zero, which is asserted.

The scales were originally applied to **gross** income, and this module described
the result as an unsolved circularity: the deduction depends on net income, but net
income depends on the deduction. Both halves of that were wrong.

* **The column is `Reineinkommen` — net income — not gross.** Keying on gross
  understated the deduction for exactly the households it exists for, since a
  household with more deductions has *lower* net income and the scale allows it
  more. Schaffhausen at CHF 20,000 with a CHF 1,500 premium moves 3,225 → 3,525;
  Fribourg at CHF 25,000 moves 3,100 → 3,500.
* **There is no circularity.** The base is income minus the *other* deductions,
  excluding the means-tested deduction itself. Subtracting that too would lower the
  base, raising the deduction, lowering the base again — a loop that only exists if
  you define the base that way. Defined as "income remaining after the other
  deductions" it is a single pass, which `means_tested_deduction_does_not_feed_its_own_base`
  asserts by assessing twice and requiring the same answer.

The distinction matters because it is the difference between a definition and a
fixed-point search, and the module previously recorded the wrong one as an
acknowledged limitation. A sweep over every canton with a scale
(`means_tested_scales_use_net_income_for_every_canton`) checks the direction, and
skips the capped regime — Valais allows CHF 21,250 against a CHF 15,000 income, so
there the base has no observable effect and asserting one would be asserting a
coincidence.

### Status: both models are printed; the tax base is not switched

`tax.rs` still uses the ~35%-cap estimate for the figures the tool reports, whose
components (`commuting = 1.5%`, `rent = 12%`, …) are invented rather than sourced.
The sourced model in `src/deductions.rs` is tested and is now **printed beside the
estimate** by `optimize`, so the difference is visible on a real scenario:

```text
🧾 DEDUCTION MODEL COMPARISON
  Gross income: CHF 100000   (ZH ESTV scale x Steuerfuss)
  Estimated (in use): CHF 34300 (34.3%)
  Sourced (federal):  CHF 12600 (12.6%)
  Sourced (ZH):       CHF 12600 (12.6%)
        6800.00  [Bund] Kinderabzug
        2800.00  [Bund] Verheiratetenabzug
        ...
```

Pointing the reported figures at the sourced model is a separate decision, because
it moves every number the tool produces.

Two properties of the estimate are worth recording because both are easy to
misread from the output:

* **The 35% cap binds for families at moderate incomes.** At CHF 60,000 with two
  children, the plain and `--family-tax-mode` figures are both CHF 21,000 — the cap
  — so the flag has no visible effect there. It works (+CHF 3,000) wherever the cap
  does not bind, e.g. CHF 60,000 with one child or CHF 140,000 with two.
* **One invented figure has been withdrawn from the output.**
  `TaxDeductionBreakdown::non_deductible_total` is 2% of income *after* the
  deductible items. It has no source, and nothing reads it — it reached the user
  only through `display.rs`, printed between "Total deductible" and "Taxable income
  after deductions" where it read as part of that arithmetic even though taxable
  income is gross minus `deductible_total` alone. A reader could not reconcile the
  lines. §7's standard is that numbers be traceable, so an untraceable one is better
  withdrawn than shown unexplained; the field is retained for serialised
  compatibility and `non_deductible_total_feeds_no_tax_figure` pins that it stays
  out of every calculation.

Until then, treat the current deduction figures as an estimate, and prefer
`--custom-tax-rate` with your observed rate.

---

## 4a. The deduction engine, and what it found

`src/deductions.rs` assesses a `Household` against a jurisdiction's imported
rules. The arithmetic is trivial; the work is **selection**.

`estv_deductions_data` holds **574 deduction rules per jurisdiction** — every
deduction a canton offers. A taxpayer takes a *subset*. Summing all 574 would
understate taxable income far more badly than the 35% cap overstates it, so
selection is explicit:

* each rule is assigned a `RuleCategory` from a **closed** list;
* a `Household` states the facts a category depends on, and fields that are
  `None` mean *unknown*, not `false`;
* a category whose facts are unknown is skipped, not assumed;
* **mutually exclusive variants are grouped**, and at most one is applied;
* anything unclassified is **reported, never applied**.

### Status of the objective

| Piece | State |
|---|---|
| Data imported with per-rule provenance | **done** — 617 rules, 22 scales, 27 jurisdictions |
| Rule kinds modelled (`Betrag`/`Prozent`/`Min`/`Max`, threshold scales) | **done** |
| Federal / cantonal composition | **done** — assessed per jurisdiction, itemised with its source |
| Selection: which rules apply to a household | **done** — with 9 bugs found and fixed |
| Coverage of the facts that gate them | **done** — property, pensioner, insurance, plus CLI flags |
| *Replacing* the estimate in the reported figures | **not done — the user's decision** |

The last row is why this objective is not closed. The sourced layer exists, is
tested, and is reachable — `--custom-tax-rate`, `--imputed-rental-value`,
`--pensioner` and `--insurance-premiums` all feed it — but `tax.rs` still computes
the reported figures from the ~35%-cap estimate. Switching that default moves every
number the tool produces, so it is a decision to be taken after reading the
comparison output rather than a defect to be corrected.

### Nine bugs the comparison found

Each was a plausible-looking number rather than an error, and each was found by
printing the engine's output beside the existing estimate — not by reasoning.
All nine are now regression tests in `tests/deduction_engine.rs`.

| Bug | Effect at CHF 100,000 |
|---|---|
| `Verheiratetenabzug` applied to a single taxpayer | CHF 2,800 too much |
| All three child age brackets summed | 37.4% of income deducted |
| Secondary-employment expenses with no secondary income | CHF 2,400 too much |
| Pillar 3a tied *and* untied variants summed | CHF 7,056 deducted twice |
| `contains("Kind")` matched `"ohne Kind"` | disabled FR's and VS's single-person deductions entirely |
| Both property-maintenance age bands summed | CHF 6,000 of maintenance against a CHF 20,000 rental value |
| `Abzug Vermögensverwaltungskosten` applied to income | 0.2–0.3% of a salary in ZH, SZ, OW, NW, GL |
| Insurance-premium rules deducted nothing | the family did nothing in all 27 jurisdictions |
| A zero-amount rule was dropped silently | a family looked like it did not exist |

The `"ohne Kind"` case is the most instructive: `"ohne Kind"` contains `"Kind"`,
so a keyword test demanded children for a scale that explicitly excludes them. The
child conditions had to be parsed as *conditions* rather than searched for as
keywords — the same class of mistake as reading `Kantons-Id` as `Kanton`.

Two of the seven share a shape worth naming, because it has now bitten four times:
**the export lists alternatives as separate rows**. Child age brackets, pillar 3a
solution type, property age bands and pensioner groups are all "pick one", and a
model that sums rows instead of grouping variants overstates every family that has
them.

### What the two models actually produce

At CHF 100,000 gross, for a household that declares nothing optional:

| Jurisdiction | Household | Sourced | Estimate |
|---|---|---|---|
| `Bund` | single | CHF 3,000 (3.0%) | CHF 15,700 (15.7%) |
| `Bund` | married, 2 children | CHF 12,600 (12.6%) | CHF 15,700 (15.7%) |
| `ZH` | single | CHF 3,300 (3.3%) | CHF 15,700 (15.7%) |
| `ZH` | married, 2 children | CHF 12,600 (12.6%) | CHF 28,300 (28.3%) |
| `AG` | single | CHF 3,000 (3.0%) | CHF 15,700 (15.7%) |
| `AG` | married, 2 children | CHF 15,400 (15.4%) | CHF 28,300 (28.3%) |

**The estimate is roughly five times the sourced figure for a single earner.** It
overstates deductions, so it *understates* tax — an error in the taxpayer's
favour, which is part of why it survived this long.

The sourced figures are lower for a real reason rather than an omission: this
household declares no insurance premiums, no pillar 3a contribution, no childcare
and no commuting costs, and the engine refuses to invent them. Supplying them
raises the total — with all four declared plus a second income, the federal total
at CHF 100,000 for a married household with two children is CHF 42,056 (42.1%),
where pillar 3a, childcare, commuting and the second-earner deduction each appear
itemised.

### Deliberate limitations

Stated rather than hidden, because each either under- or over-states the tax:

* **Not applied**, and reported by `skipped()`: self-employment, and any rule whose
  trigger `Household` has no field for.
* **Wealth deductions are not modelled at all**, because the model has no wealth.
  `Abzug Vermögensverwaltungskosten` sits in the ESTV file under
  `Steuerart = Einkommen`, which is how it came to be deducted from *income* in
  ZH, SZ, OW, NW and GL — 0.2–0.3% of a salary taken as a wealth-management cost.
  It is now reported as not applicable instead.
* **A percentage pensioner rule is not applied.** Basel-Landschaft's is 40–60%
  *of the pension*, and deducting a share of a salary instead would be a share of
  the wrong base. Only the flat-amount form is used.
* **Bracket selection within a family** takes the larger amount when the
  distinguishing fact (a child's age, a building's age, whether the taxpayer has a
  tied pension solution) is not held. The alternative is reported as skipped, so
  the uncertainty is visible rather than silent.
* **Means-tested scales** are keyed on income *net* of the other deductions, which
  is what the export's `Reineinkommen` column means. See the phase-out section
  above; the base deliberately excludes the means-tested deduction itself, which
  makes it a single pass rather than a fixed-point search.
* Totals are capped at gross income. Valais's phase-out table legitimately allows
  CHF 21,250, which exceeds a CHF 15,000 income; the cap applies to the total, and
  `uncapped_total()` exposes the difference.

### Property, and why the base matters more than the rate

Every property rule in the export is a percentage **of the imputed rental value**,
not of income: `Abzug vom Eigenmietwert` is 20–40% of it and maintenance 10–20%.
The first version of the engine applied them to gross income, which is a category
error rather than a rounding difference — a CHF 100,000 earner with a CHF 20,000
rental value would have had maintenance calculated off their salary.

`Household::homeowner(imputed_rental_value)` supplies the fact, and three things
follow:

1. **A tenant gets nothing**, because `None` means the fact is absent and every
   property rule is gated on it.
2. **The deduction scales with the home, not the salary** — asserted directly, by
   doubling each in turn and checking which one moves the figure.
3. **Property deductions are capped at the rental value.** You cannot deduct more
   maintenance against a home than the home is deemed to earn.

The `Eigenmietwert` is also an **income addition**, not only a deduction base: the
rental value the owner would otherwise have paid themselves is added to taxable
income first. Modelling only the deduction understates a homeowner's taxable
income, so `DeductionAssessment::income_addition` exposes it and
`taxable_income_with_addition()` applies both sides. At CHF 100,000 with a
CHF 20,000 rental value, taxable income lands *above* the salary despite the
deductions — which is the correct direction and the one a one-sided model gets
wrong.

### Pensioner deductions

`Abzug für AHV/IV-Rentner` is 10 flat rules plus 15 phase-out scales, across eight
cantons. `Household::pensioner()` supplies the fact. The default is `false`, not
"unknown", precisely because granting these by omission would be a large silent
error for every working household.

They were previously classified as *means-tested*, which excluded them from every
household and would have applied the wrong mechanism to a pensioner one. They are
now their own category, with the flat-amount form applied (SZ 4,000, GL 2,100,
SO 5,000) and the percentage form left alone for the reason above.

### Supplying the facts, and what happens when you do not

The comparison is driven by CLI flags, so the engine's coverage is reachable
rather than only testable:

| Flag | Fact | Effect |
|---|---|---|
| `--imputed-rental-value` | the home's `Eigenmietwert` | adds it to taxable income *and* unlocks every property deduction |
| `--pensioner` | receives an AHV/IV pension | unlocks 10 rules and 15 scales, otherwise never applied |
| `--insurance-premiums` | declared premiums | deducted up to the canton's ceiling |

A fact that is **not** supplied is skipped rather than assumed. That is why an
unadorned run shows a small sourced total: it is reporting what it was told, and
the trailing "N further rule(s) were considered and not applied" line is the
measure of what the household did not declare. Supplying all three moves a
CHF 100,000 married Zurich household from CHF 3,000 to roughly CHF 14,000 of
federal deductions.

### Two more bugs found while wiring the flags

Both are regression tests now, and the second is the more instructive:

| Bug | Effect |
|---|---|
| Insurance-premium rules deducted nothing | the whole family did nothing, across all 27 jurisdictions |
| A rule yielding zero was dropped silently | a whole family was indistinguishable from one that does not exist |

The insurance rules state **only a ceiling** — `Betrag = 0`, `Prozent = 0`,
`Maximum = 5800` — so the generic `clamp(amount + percent × base, …)` returned
zero for every one of them. The second bug is why that was invisible: the engine
discarded zero-amount rules with a silent `continue`, so the family vanished from
both the applied *and* the skipped list. A dropped rule cannot be distinguished
from a rule that does not exist, which is the exact failure the `skipped` list was
built to prevent — and it had a hole in it.

### Two more collisions

Both were plausible numbers rather than errors, and both are now regression tests:

| Bug | Effect |
|---|---|
| Both property-maintenance age bands summed | CHF 6,000 of maintenance against a CHF 20,000 rental value where the law allows one band |
| `Abzug Vermögensverwaltungskosten` applied to income | 0.2–0.3% of a salary in ZH, SZ, OW, NW, GL |

The pattern is now familiar enough to name: **the export lists alternatives as
separate rows**, and a model that sums rows instead of grouping variants will
overstate every family that has them. It has now bitten four times — child age
brackets, pillar 3a solution type, property age bands, and pensioner groups.

---

## 5. What `--canton` does today

`--canton` is wired to the tax data, and behaves in one of three explicit ways:

| Invocation | Behaviour |
|---|---|
| `--canton` omitted | Uses **Bern** (the historical default) and prints that it did so |
| `--canton BE` | Uses Bern's standalone Stadt Bern table |
| `--canton XX` (unknown code) | Exits 2, listing the valid codes |
| `--canton ZH` (known, priced) | Uses the ESTV scale × Steuerfuss, and names the basis |
| `--canton VS` (known, not priced) | **Exits 2** naming exactly what is missing |
| `--custom-tax-rate 0.14` | Bypasses canton resolution entirely |

This replaced a genuinely misleading behaviour: `--canton` was accepted,
defaulted to `"ZH"`, and **silently ignored** — so `--canton ZH` produced Bern
numbers. A flag that appears to change the answer but does not is worse than an
absent flag, because the user has no way to notice.

The failure message for an unpriced canton names only what is genuinely absent,
and lists the cantons that *are* priced. The list is derived rather than
hard-coded: the count went from 3 to 22 during this project, and a stale hint
would send the reader away from a canton that works. It also no longer claims the
scale is missing for the four cantons whose scale is imported and whose
*multiplier* is the blocker:

```
error: cannot price canton VS (Valais). Missing: cantonal Steuerfuss.

  Cantonal tax = simple_tax(income) x Steuerfuss, where both sides
  must come from the same year. For VS: cantonal Steuerfuss
  The multiplier is read from the ESTV Steuerfuss workbook, whose 2026 cell is
  not a plain number (blank, or a footnote this project will not guess at).

  Two ways forward:
    - use a canton that is priced (22): AG, AR, AI, BE, BS, GL, GR, JU, LU, NE, NW, OW, SH, SZ, SO, SG, TG, TI, UR, VD, ZG, ZH
    - pass --custom-tax-rate with your observed rate from your tax assessment (most accurate)

  See SWISS_TAX_DATA.md §3 for the table format needed.
```

`tests/cli_canton.rs` pins the table's behaviour, including that no projection is
printed alongside the error.

---

## 6. A rate-base defect in the reported figures

Found while auditing the two deduction models against each other, and unrelated to
either: `TaxSchedule` had **two accessors that disagreed about which income the rate
schedule applies to**.

```rust
effective_tax_rate(gross)  ->  looked up on taxable_income_after_estimates(gross)  ✅
tax_only_rate(gross)       ->  looked up on gross                                  ❌
```

`tax_only_rate` supplies the **printed "Tax Rate"**, so the tool reported a rate
that was not the rate it charged. The tax itself was always computed on deducted
income, which is correct — only the number shown to the user was wrong, by:

| Gross income | Rate on gross (shown) | Rate on taxable (charged) | Overstated by |
|---|---|---|---|
| CHF 50,000 | 12.00% | 10.56% | 1.44pp |
| CHF 100,000 | 17.24% | 15.76% | 1.48pp |
| CHF 150,000 | 21.34% | 19.50% | 1.84pp |
| CHF 250,000 | 24.73% | 24.73% | 0.00pp |

The gap closes at CHF 250,000 because the Bern table clamps above CHF 200,000 —
which is precisely why the defect could sit at the *top* of the range unnoticed.

**How it stayed hidden.** Three tests called `tax_only_rate` with a *taxable*
figure and asserted the published rates. They passed only because the function
deducted a second time and the test then compared against a band that tolerated the
result. The bug and its tests agreed with each other, so neither looked wrong.
`bern_table_is_close_to_the_published_figures` is the case in point: it asserted
`|diff| < 15.0` where the real answer is −5.74pp.

**The fix.** `tax_only_rate(gross)` now deducts first, so both accessors use one
base and agree to within 1e-12 — asserted by
`rate_accessors_agree_on_the_same_income`. A new `tax_rate_on_taxable(income)` is
the way to ask about an income that is already taxable, which is what the
published-reference tests actually wanted; converting a taxable figure back to a
gross one just to have it deducted again is the mistake that hid this.

**What did not change.** The deduction *model* is untouched — still the ~35%-cap
estimate, still with the sourced model printed beside it. This fix makes the
reported rate consistent with the tax already being charged, rather than moving any
figure between models. That it restores the documented Bern table exactly is the
evidence it is a fix and not a recalculation.

---

There are two honest paths today:

1. **`--custom-tax-rate`** — pass your own observed effective rate as a decimal
   (e.g. `0.1382` for 13.82%), taken from your tax assessment. This bypasses the
   canton tables entirely and is the most accurate option available, because it
   uses your actual figure rather than a modelled one.
2. **Bern** — `TaxSchedule::bern_city_default` carries the original hand-entered
   Stadt Bern table (cantonal + municipal + church combined), sourced from the
   city's own `Steuerbelastung des Arbeitseinkommens` publication.

---

## 7. What works in the meantime

The two honest paths, unchanged by the work above:

1. **`--custom-tax-rate`** — pass your own observed effective rate as a decimal
   (e.g. `0.1382` for 13.82%), taken from your tax assessment. This bypasses the
   canton tables entirely and is the most accurate option available, because it
   uses your actual figure rather than a modelled one.
2. **Bern** — `TaxSchedule::bern_city_default` carries the original hand-entered
   Stadt Bern table (cantonal + municipal + church combined), sourced from the
   city's own `Steuerbelastung des Arbeitseinkommens` publication.

---

---

## 8. FIXED: tax was charged on gross income, not taxable income

**Found, verified, and NOT yet fixed.** Recorded here so it is not lost, because it
changes the tool's central calculation and needs its own deliberate pass.

`TaxSchedule` looks the burden rate up on **taxable** income (corrected in section 6)
and then multiplies it by **gross** income:

```rust
let taxable = self.taxable_income_after_estimated_deductions(gross_income);
let base_tax_rate = self.lookup_tax_rate(taxable);      // rate IS a burden rate
...
gross_income * (1.0 - (base_tax_rate + social))         // applied to GROSS -- the defect
```

### Why the rate is a burden rate

The Bern table's own reference points confirm it: 10.08% at CHF 40,000 means the
burden **on CHF 40,000 taxable** is CHF 4,032, and the model reproduces each
published point exactly -- which is what
`tests/published_tax_reference.rs::bern_table_is_close_to_the_published_figures`
pins. So the rate is tax / taxable, a function of taxable income. Multiplying it by
gross income overcharges.

### The fix

```text
tax            = rate(taxable) x taxable
effective_rate = tax / gross + social
after_tax      = gross x (1 - effective_rate)      // exact inverse
```

At CHF 100,000 gross, single, Bern: taxable CHF 84,300, rate 15.76%.

| Quantity | Before | After |
|---|---|
| Tax charged | CHF 15,758 | **CHF 13,284** |
| Tax share of gross | 15.76% | 13.28% |
| Effective rate (tax + 12.9% payroll) | 28.66% | 26.18% |
| After-tax income | CHF 71,341.60 | **CHF 73,815.67** |
| Overcharge removed | | **CHF 2,474** |

At CHF 70,000 gross the tax falls from CHF 9,314.51 to CHF 7,852.14 and after-tax
income rises from CHF 51,655.49 to CHF 53,117.86.

### What it changed, and why it needed its own pass

`after_tax_income` feeds `monthly_after_tax`, which decides **feasibility** and the
optimal work percentage — so this moved recommendations, not merely a displayed
number, exactly as anticipated when it was recorded rather than patched.

All 202 tests were re-run. The three that failed were asserting the *old* relation
(`effective == tax_only + social`, which held only while the tax was charged on
gross); they now assert the burden decomposition against the schedule rather than in
the scenario's own terms, so they cannot pass by both sides moving together.

### A correction to the previous round's report

The claim "25.6% vs ~20%" compared the estimate's *share of gross* against the
sourced model's *deduction share of gross* -- two different quantities. Measured
consistently, the divergence is much smaller: over 20 household/income
combinations, the reported burden rate differs by **0.3 to 2.1 percentage points**,
and by 0.0pp once the Bern table clamps at CHF 250,000.

---
## 9. Current status summary

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
