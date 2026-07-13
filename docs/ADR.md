# Architecture Decision Record — rstree

> Persisted architectural context for `rstree`. Also stored in the
> `codebase-memory-mcp` project cache (`manage_adr`). This file is the
> git-shareable copy.

## PURPOSE
rstree is a lightweight, dependency-free clone of the Unix `tree` command. Given an optional directory path (defaulting to the current directory), it recursively prints the directory hierarchy using classic `tree` box-drawing characters (`├──`, `└──`, `│   `).

## STACK
- Language: Rust; no external runtime dependencies — standard library only (no third-party crates in Cargo.toml).
- Build/test: cargo; CI via `.github/workflows/ci.yml`.
- Packaging: single binary crate (`rstree`) with a library surface in `src/lib.rs`.

## ARCHITECTURE
- Binary crate with a thin entry point (`src/main.rs`) and a fat library (`src/lib.rs`). `main` parses arguments and prints the summary/error lines; all tree logic lives in `lib`. Boundary: `main -> lib` (single call site).
- Core API: `generate(root: &Path, out: &mut impl Write, opts: &Options) -> io::Result<TreeStats>`. The render sink is a generic `io::Write`, decoupling rendering from I/O.
- `Options { show_hidden: bool, max_depth: Option<usize> }` controls behavior; `TreeStats { directories, files }` is returned for the CLI summary line ("N directories, M files").
- `walk()` performs recursive descent, building prefix strings per level and emitting connectors. `root_display_name()` resolves a friendly root label (e.g. the real folder name when the caller passed `.`).

## PATTERNS
- Recursive directory walk with explicit prefix scaffolding (manual construction of `├── `/`└── ` connectors and `│   `/`    ` child prefixes) rather than a generic pretty-printer; output matches classic `tree`.
- Deterministic, locale-independent ordering: entries sorted by file name; hidden entries (names starting with `.`) excluded by default to mirror `tree`.
- Symlink safety: symlinks are never descended into; rendered as `name -> target` to avoid infinite loops on symlink cycles.
- Graceful degradation: mid-traversal read errors print `[Access Denied]`/`[Error: ...]` and the walk continues; root-level permission denied yields a minimal single-line tree (`TreeStats { directories: 1, files: 0 }`); a non-directory root returns `InvalidInput` (surfaced by the CLI as a friendly message).
- Two-tier testing: in-crate unit tests (`src/lib.rs` `#[cfg(test)]`) drive `generate` against in-memory `Vec<u8>` buffers using a std-only `Tmp` temp dir (Drop-based cleanup); integration tests (`tests/cli.rs`) spawn the compiled binary via `CARGO_BIN_EXE_rstree` to verify real CLI behavior.

## TRADEOFFS
- Zero dependencies keeps the binary tiny and trivially installable, at the cost of hand-rolled temp-dir management and manual cross-platform symlink handling (`#[cfg(unix)]` / `#[cfg(windows)]`).
- Custom prefix scaffolding is simple and output-exact but less reusable/general than a pluggable pretty-printer.
- The std-only `Tmp`/`make_tmp` helpers are duplicated between `lib.rs` tests and `tests/cli.rs`; accepted to avoid adding a dev-dependency.
- Permission-denied tests skip (rather than fail) when run as root, since the assertions cannot hold in that context.

## PHILOSOPHY
- Standard library only; no external dependencies.
- Rendering must be testable without a terminal or filesystem (generic `io::Write` sink).
- Fail soft on individual path errors; only root-level, structurally invalid input (missing path / not a directory) is a hard error surfaced to the user.
- Deterministic, `tree`-compatible output is a deliberate goal.
