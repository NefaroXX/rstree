# ls-tree

[![CI](https://github.com/NefaroXX/ls-tree/actions/workflows/ci.yml/badge.svg)](https://github.com/NefaroXX/ls-tree/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A lightweight clone of the Unix `tree` command, written in Rust. Point it at a
directory and it recursively prints the folders and files as a visual hierarchy
with optional JSON output, file sizes, colouring, sorting, and more.

```
$ ls-tree ./src
src
├── lib.rs
└── main.rs

2 directories, 2 files
```

## Features

- Classic `tree` output using `├──`, `└──`, and `│   ` connectors.
- **JSON output** (`--json`) — machine-readable for AI shells and OS tools.
- **File sizes** (`-s`/`--size`) with **human-readable** (`-h`) formatting.
- **Colour support** — directories, executables, and symlinks are ANSI-coloured
  automatically (respects `NO_COLOR`, auto-detects terminal).
- **`.gitignore` awareness** (`--git-ignore`) — filters entries matching
  root-level `.gitignore` rules.
- **Sorting** (`--sort=name/size/time`) — order entries within each directory.
- **Prune** (`--prune`) — omit empty directories from the output.
- **Directories only** (`--dirs-only`) — hide regular files.
- **File-type icons** (`--icons`) — Unicode icons for directories, symlinks,
  and file types.
- **Total size** (`--total-size`) — show recursive directory sizes.
- Robust error handling — unreadable directories print `[Access Denied]`
  and the traversal continues instead of crashing.
- Symlink-safe — symlinks are shown (with their target) but never descended
  into, so symlink cycles cannot hang the program.
- Hidden-file aware — dotfiles are hidden by default (like `tree`), with a
  flag to reveal them.
- `--max-depth` to limit how far down the tree is printed.

## Installation

### From source (recommended)

```bash
git clone https://github.com/NefaroXX/ls-tree.git
cd ls-tree
cargo install --path .
```

### From crates.io

```bash
cargo install ls-tree
```

### Build locally

```bash
cargo build --release
./target/release/ls-tree
```

## Usage

```bash
ls-tree [OPTIONS] [PATH]
```

If `PATH` is omitted, `ls-tree` prints the current directory (`.`).

### Options

| Flag | Description |
| --- | --- |
| `-a`, `--all` | Show hidden entries (names starting with `.`). |
| `-L`, `--max-depth N` | Descend at most `N` directory levels. |
| `--dirs-only` | Display directories only. |
| `-s`, `--size` | Show file sizes. |
| `-h`, `--human-readable` | Print sizes in human-readable format (implies `--size`). |
| `--json` | Output JSON instead of the classic tree. |
| `--git-ignore` | Respect `.gitignore` rules. |
| `--sort=<field>` | Sort entries by `name` (default), `size`, or `time`. |
| `--prune` | Omit empty directories from the output. |
| `--icons` | Show Unicode file-type icons. |
| `--total-size` | Show recursive directory sizes (implies `--size`). |
| `--color` | Force colour output (auto-detected by default). |
| `--no-color` | Disable colour output. |
| `--help` | Print the help message and exit. |
| `-V`, `--version` | Print version information and exit. |

### Examples

```bash
# Current directory
ls-tree

# Specific path
ls-tree ./src

# Include hidden files, two levels deep
ls-tree -a -L 2 /etc

# JSON output with sizes
ls-tree --json --size ./src

# Git-ignore aware, sorted by size
ls-tree --git-ignore --sort=size

# Human-readable sizes with prune and icons
ls-tree -h --prune --icons
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success (including when some subdirectories were unreadable). |
| `1` | The path does not exist, is not a directory, or could not be accessed. |

## How it works

`ls-tree` reads the provided path, then recursively walks the directory tree with
`std::fs::read_dir` and `std::path::PathBuf`, emitting an indented hierarchy.
Strings and paths are passed through the recursive walk as borrowed `&Path`
values with owned `PathBuf`s only where ownership is genuinely needed, keeping
the borrow checker happy.

Edge cases handled explicitly:

- **Hidden files / folders** — skipped by default, shown with `-a`.
- **Restricted permissions** — an unreadable directory prints `[Access Denied]`
  inline and traversal continues.
- **Symlink cycles** — symlinks are displayed but never descended into.

## Development

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and please
follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under the [MIT License](LICENSE).
