# Unruled Examples

This directory contains temporary, experimental examples that are **exempt**
from the project's syntax and architecture rules in `.agents/rules/`.

These exist to quickly test new functionality, network integration, and
proof-of-concept code without being constrained by the atomic file structure,
decomposition, `#[rustfmt::skip]`, or other conventions.

**Only this folder** is exempt. All other code in `src/`, `examples/`, etc.
must continue to follow `.agents/rules/` strictly.

Once an experiment stabilizes, it should be refactored to comply with the
rules before moving into `examples/` or `src/`.
