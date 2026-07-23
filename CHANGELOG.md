# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Repository community-health files: `SECURITY.md`, issue templates, and a pull request template.

## [0.2.0] - 2026-07-23

### Added
- **JSON output** (`--json`): machine-readable tree serialization.
- **File sizes** (`-s`/`--size`): show sizes next to each entry.
- **Human-readable sizes** (`-h`): format sizes with KiB/MiB suffixes.
- **ANSI colours**: automatic colouring for directories (bold blue),
  executables (green), and symlinks (cyan). Respects `NO_COLOR` and
  auto-detects terminal output.
- **`.gitignore` support** (`--git-ignore`): filter entries matching root-level
  `.gitignore` rules (via the `ignore` crate).
- **Sorting** (`--sort=name/size/time`): order entries within each directory.
- **Prune** (`--prune`): omit empty directories from output.
- **Directories only** (`--dirs-only`): hide regular files.
- **File-type icons** (`--icons`): Unicode icons for directories, files,
  symlinks, and common extensions.
- **Total directory sizes** (`--total-size`): recursive size per directory.
- New `--color`/`--no-color` flags to override terminal detection.
- `--help` (long form only); `-h` now used for human-readable sizes.

### Changed
- Dependencies introduced: `serde`, `serde_json`, `ignore`.
- Tree is now built in memory before rendering (enables JSON/prune/sorting).
- Help (`-h`) flag replaced with `--help` only (breaking: `-h` is now
  `--human-readable`).

### Removed
- Zero-dependency constraint (ADR updated to reflect new philosophy).

## [0.1.0] - 2026-07-13

### Added
- Initial release: a lightweight, dependency-free `tree` clone.
- Recursive directory printing with classic `├──` / `└──` / `│   ` connectors.
- Optional path argument (defaults to the current directory).
- `-a` / `--all` to show hidden (dot) entries.
- `-L` / `--max-depth N` to limit traversal depth.
- `-h` / `--help` and `-V` / `--version` flags.
- Graceful handling of unreadable directories (`[Access Denied]`) and other
  read errors, without crashing.
- Symlink-safe traversal (symlinks are shown with their target but never
  descended into, preventing cycles).
- Final directory/file summary line.
