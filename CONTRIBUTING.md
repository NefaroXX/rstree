# Contributing to rstree

Thanks for your interest in improving `rstree`! This project aims to stay small,
fast, and **dependency-free**. A few guidelines keep it that way.

## Getting started

1. Fork the repository and clone your fork.
2. Make sure you have a recent stable Rust toolchain (`rustup toolchain install stable`).
3. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Before opening a pull request

Run the same checks CI runs, and make sure they pass locally:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

- **Formatting:** code must be `rustfmt`-clean.
- **Lints:** `clippy` runs with `-D warnings` — no warnings allowed.
- **Tests:** add or update tests for any behaviour change. Unit tests live in
  `src/lib.rs`; binary/CLI tests live in `tests/`.

## Design principles

- **Standard library only.** Do not add external crate dependencies without an
  explicit, discussed reason. The whole point of `rstree` is being a tiny,
  dependency-free `tree` clone.
- **Graceful, never panicking.** Unreadable paths, broken symlinks, and odd
  filesystems should produce a clear inline message (`[Access Denied]`,
  `[Error: …]`), not a crash.
- **Small, focused changes.** Keep PRs focused on one improvement. Update
  `CHANGELOG.md` under the `[Unreleased]` heading for user-facing changes.

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) where
practical, e.g. `feat:`, `fix:`, `docs:`, `test:`, `refactor:`.

## Code of Conduct

By participating, you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).
