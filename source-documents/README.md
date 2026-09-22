# Source documents

The external publications this repository's Swiss tax numbers come from.

`src/` holds only *generated* Rust tables. The documents those tables were
generated from live here, so that a figure in the code can be traced back to the
publication that contains it — which is the standard `FutureWork.md` §7 sets and
`SWISS_TAX_DATA.md` applies.

**These are inputs, not outputs. Nothing in `src/`, `tests/`, or the build reads
them.** Regeneration is a deliberate manual step, so a change here cannot silently
change a recommendation; it can only change what the next regeneration produces.
`SWISS_TAX_DATA.md` records what each generated table contains and where the
remaining gaps are.

Everything in this directory is committed. Together the files are about 7.9 MB.

---

## 1. Steuerfüsse — the cantonal and municipal multipliers

Official ESTV publications, *"Steuerfüsse in den Kantonshauptorten"*. These supply
the second level of the two-level model: the simple tax is multiplied by a
cantonal and a municipal Steuerfuss.

| File | What it is | Consumed by |
| --- | --- | --- |
| `steuerfuesse-np-1995-2026.xlsx` (182 KB) | Income and wealth taxes of **natural persons**, 1995–2026 | `tools/generate_steuerfuss.py` → `src/canton_steuerfuss_data.rs` (832 rows: 26 cantons × 32 years) |
| `steuerfuesse-jp-1995-2026.xlsx` (174 KB) | Profit and capital taxes of **legal persons** | **Nothing.** See the caveat in §5 |

## 2. ESTV "Tarife" exports — the cantonal simple-tax scales

One workbook per canton, exported from the ESTV tax calculator's "Tarife"
section. Each holds the simple-tax brackets (`Für die nächsten CHF` band widths
and marginal rates) plus the splitting factor and the `Steuersubjekt` values that
express marital treatment.

| File | Canton | Consumed by |
| --- | --- | --- |
| `estv_scales_AG.xlsx` | Aargau | `tools/import_estv_scales.py` → `src/estv_scales_data.rs` |
| `estv_scales_AI.xlsx` | Appenzell Innerrhoden | same |
| `estv_scales_AR.xlsx` | Appenzell Ausserrhoden | same |
| `estv_scales_BL.xlsx` | Basel-Landschaft | same — see the note in §5 |
| `estv_scales_BS.xlsx` | Basel-Stadt | same |
| `estv_scales_FR.xlsx` | Fribourg | same |
| `estv_scales_GE.xlsx` | Genève | same |
| `estv_scales_GL.xlsx` | Glarus | same |
| `estv_scales_GR.xlsx` | Graubünden | same |
| `estv_scales_JU.xlsx` | Jura | same |
| `estv_scales_LU.xlsx` | Luzern | same |
| `estv_scales_NE.xlsx` | Neuchâtel | same |
| `estv_scales_NW.xlsx` | Nidwalden | same |
| `estv_scales_OW.xlsx` | Obwalden | same |
| `estv_scales_SG.xlsx` | St. Gallen | same |
| `estv_scales_SH.xlsx` | Schaffhausen | same |
| `estv_scales_SO.xlsx` | Solothurn | same |
| `estv_scales_SZ.xlsx` | Schwyz | same |
| `estv_scales_TG.xlsx` | Thurgau | same |
| `estv_scales_TI.xlsx` | Ticino | same |
| `estv_scales_UR.xlsx` | Uri | same |
| `estv_scales_VD.xlsx` | Vaud | same |
| `estv_scales_VS.xlsx` | Valais | same |
| `estv_scales_ZG.xlsx` | Zug | same |
| `estv_scales_ZH.xlsx` | Zürich | same |
| `estv_scales_Bund.xlsx` | **Federal** direct tax | `src/federal_tariff_data.rs` — reconciled against the statute in §3 |

25 cantonal exports plus the federal one. **There is no `estv_scales_BE.xlsx`:**
Bern has no export in this set, which is why `src/estv_scales_data.rs` lists 26
files and `BE` is not among them. `tools/diagnose_exports.py` knows this and prints
the cantons it never saw.

## 3. Deduction rules and phase-out scales

| File | What it is | Consumed by |
| --- | --- | --- |
| `estv_deductions.xlsx` (28 KB) | One rule per canton per deduction category: `Betrag`, `Prozent`, `Minimum`, `Maximum`. 618 rows → 617 merged across 27 jurisdictions | `tools/import_estv_deductions.py` → `src/estv_deductions_data.rs` |
| `estv_deduction_scales.xlsx` (21 KB) | Phase-out tables for the means-tested deductions: `Reineinkommen` → `Abzug CHF`. 466 rows → 22 scales | same |

## 4. Statute and tariff PDFs

| File | What it is | Referenced by |
| --- | --- | --- |
| `SR 642.11.pdf` (3.2 MB) | The **DBG** (*Bundesgesetz über die direkte Bundessteuer*), Systematische Rechtssammlung 642.11. The statute behind the federal tariff | The doc comments in `src/federal_tax.rs`, and `SWISS_TAX_DATA.md` §"Federal tariff". **Art. 36** was the article reconciled against `estv_scales_Bund.xlsx` |
| `progression-de-fr.pdf` (0.8 MB) | ESTV / AFC *Steuermäppchen* on **progression** ("Kalte Progression"), Team Steuerdokumentation, dated 11.11.2025, for the 202x tax period | **Nothing.** See §5 |
| `Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf` (3.2 MB) | A tariff table headed *"Einkommenssteuer (Art. 42 Abs. 1 – Verheiratete), ab Steuerjahr 2026"*, with columns for taxable income, `Einheits-Satz [%]` and `Einfache Steuer [CHF]` | **Nothing.** See §5 |

---

## 5. Caveats — files that nothing consumes

Recorded here rather than left to be rediscovered, because an unexplained 3 MB
PDF in a repository is the kind of thing that gets deleted and then missed.

- **`steuerfuesse-jp-1995-2026.xlsx` is not read by any script.** Corporate tax is
  outside the scope of a personal work-life tool, and `tools/generate_steuerfuss.py`
  reads a single workbook with its sheet finder pinned to the `NP <year>` (natural
  persons) prefix. The file is kept for completeness. Its docstring used to list it
  as an input, which overstated the situation; it now says this explicitly, so the
  next reader does not go looking for the code path that consumes it.

- **`progression-de-fr.pdf` is not cited by any code, test, or document.** It was
  added in `45a0099` ("Addded PDF Document Kalte Progession"). Its subject — how
  bracket creep under inflation moves a taxpayer between bands — is relevant to a
  model that projects income to retirement, and nothing in this repository uses it.

- **`Tarif Art. 42 - Abs. 1 - 2026-20XX_d.pdf` is not cited by any code, test, or
  document.** It was added in `538514d` ("Fix Swiss tax model and family deductions
  for Bern scenarios"), which is the only evidence of its origin: the document
  itself does not name a canton, and no repository file mentions "Art. 42". Bern's
  *Steuergesetz* Art. 42 is the income-tax tariff ([BELEX](https://www.belex.sites.be.ch/),
  [taxinfo Bern](https://www.taxinfo.sv.fin.be.ch/taxinfo/f3ad83d5-30d6-41c9-8f79-c34e192c57ad)),
  and the adding commit is about Bern — which is suggestive, not established.
  **Identify this document from a primary source before citing it as the Bern
  tariff.**

- **`annex-25544-2.pdf` is not here and is not in the repository.** It is the
  Basel-Landschaft Vademecum page for SGS 331.2 that was supplied directly by the
  user and is discussed at length in `src/cantons.rs`. Only the *conclusion* drawn
  from it is recorded in the code; the document itself was never committed. The
  load-bearing BL source is `estv_scales_BL.xlsx`, which is here.

---

## 6. How the tooling finds these files

The importers in `tools/` accept a bare filename and resolve it against this
directory, so the commands in `SWISS_TAX_DATA.md` work from any working directory:

```bash
python tools/generate_steuerfuss.py steuerfuesse-np-1995-2026.xlsx src/canton_steuerfuss_data.rs
python tools/import_estv_scales.py src/estv_scales_data.rs estv_scales_AG.xlsx
python tools/import_estv_deductions.py src/estv_deductions_data.rs
```

An explicit path still wins, which is what the overrides exist for:

```bash
python tools/import_estv_scales.py src/estv_scales_data.rs --federal-out src/federal_tariff_data.rs estv_scales_Bund.xlsx
```

Two of the tools sweep rather than take an argument —
`tools/survey_estv_scales.py` and `tools/diagnose_exports.py` both glob
`estv_scales_*.xlsx` here — so they find the whole set without being told where it
is. `tools/pdf_text.py` and its siblings take any path.

The generated headers cite the **basename** of the input, not the resolved path:
baking a machine-specific absolute path into a committed Rust file would make the
provenance wrong for every other checkout.

This resolves the paths, it does not change the data. Moving these files was
verified by regenerating from the new location with bare filenames and diffing:
`src/estv_scales_data.rs`, `src/federal_tariff_data.rs` and
`src/estv_deductions_data.rs` come back **byte-identical**, and
`src/canton_steuerfuss_data.rs` differs only in the date stamp the generator
writes into its header.
