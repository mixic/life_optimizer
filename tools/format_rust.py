"""Format a generated Rust file, so regenerating keeps `cargo fmt --check` green.

The repository's CI runs `cargo fmt --all -- --check`. The data tables in
`src/estv_scales_data.rs`, `src/estv_deductions_data.rs`,
`src/canton_steuerfuss_data.rs` and `src/federal_tariff_data.rs` are emitted by
scripts rather than written by hand, and the emitter produces long identifiers
and long array rows that rustfmt wants to wrap. Without this step there are only
two stable states, both bad:

  * the generated files are committed as emitted, and the formatting gate fails
    on code no human wrote, or
  * the generated files are formatted by hand, and the next run of the importer
    silently reverts them, so the gate fails later for a reason nobody connects
    to the regeneration.

Running rustfmt here makes the emitters idempotent with the gate: the file that
is generated is the file `cargo fmt` would produce.

`rustfmt.toml` at the repository root widens `use_small_heuristics` so that the
one-line struct literals these tables are made of survive formatting. Without it
rustfmt expands every `FederalBracket { threshold: 0.0, rate: 0.0 }` into a
four-line block and quadruples the file.

If rustfmt is not installed the emitters still succeed, with a note: the
generated data is correct either way, and the failure is a formatting one that
`cargo fmt --all` fixes.
"""

import os
import subprocess


def format_generated(path):
    """Run `rustfmt` on `path`, returning True when it formatted cleanly."""
    path = os.path.abspath(path)
    if not os.path.exists(path):
        return False
    try:
        completed = subprocess.run(
            ["rustfmt", "--edition", "2021", path],
            capture_output=True,
            text=True,
            cwd=os.path.dirname(os.path.dirname(path)) or None,
        )
    except (FileNotFoundError, OSError):
        print(f"  note: rustfmt not available; run `cargo fmt --all` to format {path}")
        return False
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "").strip().splitlines()
        first = detail[0] if detail else "no output"
        print(f"  note: rustfmt failed on {path}: {first}")
        print("        run `cargo fmt --all` before committing")
        return False
    return True
