# Contributing to Life Optimizer

Thank you for your interest in contributing to **Life Optimizer**!  
This project aims to provide a scientifically grounded, transparent, and extensible decision engine for long-term work–life planning. Contributions are welcome across modeling, optimization, documentation, and UX.

Please read this guide before submitting issues or pull requests.

---

## Code of Conduct

By participating in this project, you agree to uphold a respectful, constructive, and inclusive environment.  
Be kind, be clear, and help others grow.

---

## How to Contribute

### 1. Reporting Issues
If you find a bug, inconsistency, or unclear documentation:

- Search existing issues first.
- Open a new issue with:
  - Clear title
  - Steps to reproduce
  - Expected vs. actual behavior
  - Relevant logs or screenshots

### 2. Suggesting Enhancements
Enhancement proposals should include:

- Motivation (what problem does it solve?)
- Proposed solution or design
- Alternatives considered
- Potential impact on architecture or performance

### 3. Submitting Pull Requests

#### Workflow
1. Fork the repository  
2. Create a feature branch  
3. Commit changes with meaningful messages  
4. Ensure code compiles and tests pass  
5. Open a pull request describing:
   - What was changed
   - Why it was changed
   - Any limitations or follow-up work

#### Requirements
- Rust code must follow `rustfmt` and `clippy` guidelines.
- All new features must include tests.
- Public APIs must include documentation comments.
- Complex algorithms should include inline explanations.

---

## Testing

Life Optimizer uses Rust’s built-in testing framework.

```bash
cargo test              # everything
cargo test --test cli   # one integration suite
cargo test conversion   # tests whose name contains "conversion"
```

### Requirements
- Unit tests for domain logic (taxes, BVG, utility functions)
- Integration tests for CLI commands
- Simulation tests validating statistical properties
- Regression tests for optimization outputs

### Layout

`src/` holds the library (`lib.rs`) plus the CLI wrapper (`main.rs`). Tests live
in two places, and the split is deliberate:

| Location | Scope | Why there |
|---|---|---|
| `src/**` `#[cfg(test)]` | Unit tests needing access to private items | `optimizer.rs` and `tax.rs` test internals; integration tests cannot reach them |
| `tests/conversion_rate.rs` | Conversion-rate contract (FutureWork §10, MATHEMATICS §6.2.1) | Pure public API |
| `tests/optimizer_behavior.rs` | Search behaviour, feasibility flag, regression tests | Pure public API |
| `tests/cli.rs` | End-to-end binary behaviour: flags, exit codes, output text | The only way to test process exit and the exact user-facing text |

A binary-only crate cannot be imported by `tests/`, which is why `src/lib.rs`
exists. `main.rs` is a thin wrapper and contains no domain logic — if you are
adding logic, it belongs in the library where it can be tested.

### Writing regression tests

When you fix a bug, add a test that fails without your fix and names the
behaviour, not the function. Existing examples to follow:

- `tests/optimizer_behavior.rs::past_retirement_age_does_not_panic` — guards a
  `u32` underflow panic.
- `tests/optimizer_behavior.rs::infeasible_search_reports_fallback_and_minimises_shortfall`
  — guards against advice that ranks unaffordable options by utility.
- `tests/cli.rs::infeasible_household_does_not_claim_an_optimal_solution` —
  guards the user-facing wording, which unit tests cannot see.

### Continuous integration

`.github/workflows/ci.yml` runs formatting, clippy, and the full test suite on
every push and pull request. Run `cargo fmt --check` and
`cargo clippy --all-targets` before opening a pull request.

---

## Project Structure

