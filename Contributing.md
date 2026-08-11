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

### Requirements
- Unit tests for domain logic (taxes, BVG, utility functions)
- Integration tests for CLI commands
- Simulation tests validating statistical properties
- Regression tests for optimization outputs

---

## Project Structure

