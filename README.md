# rstree

[![CI](https://github.com/NefaroXX/rstree/actions/workflows/ci.yml/badge.svg)](https://github.com/NefaroXX/rstree/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A lightweight, **dependency-free** clone of the Unix `tree` command, written in
Rust. Point it at a directory and it recursively prints the folders and files
as a visual hierarchy — using only the Rust standard library.

```
$ rstree ./src
src
├── lib.rs
└── main.rs

2 directories, 2 files
```

## Features

- 🌳 Classic `tree` output using `├──`, `└──`, and `│   ` connectors.
- 📦 **Zero dependencies** — pure `std` (`std::fs`, `std::path`, `std::env`).
- 🛡️ Robust error handling — unreadable directories print `[Access Denied]`
  and the traversal continues instead of crashing.
- 🔗 Symlink-safe — symlinks are shown (with their target) but never descended
  into, so symlink cycles cannot hang the program.
- 🙈 Hidden-file aware — dotfiles are hidden by default (like `tree`), with a
  flag to reveal them.
- 🎚️ `--max-depth` to limit how far down the tree is printed.

## Installation

### From source (recommended)

```bash
git clone https://github.com/NefaroXX/rstree.git
cd rstree
cargo install --path .
```

### From crates.io

```bash
cargo install rstree
```

### Build locally

```bash
cargo build --release
./target/release/rstree
```

## Usage

```bash
rstree [OPTIONS] [PATH]
```

If `PATH` is omitted, `rstree` prints the current directory (`.`).

### Options

| Flag | Description |
| --- | --- |
| `-a`, `--all` | Show hidden entries (names starting with `.`). |
| `-L`, `--max-depth N` | Descend at most `N` directory levels. |
| `-h`, `--help` | Print the help message and exit. |
| `-V`, `--version` | Print version information and exit. |

### Examples

```bash
# Current directory
rstree

# Specific path
rstree ./src

# Include hidden files, two levels deep
rstree -a -L 2 /etc
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success (including when some subdirectories were unreadable). |
| `1` | The path does not exist, is not a directory, or could not be accessed. |

## How it works

`rstree` reads the provided path, then recursively walks the directory tree with
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
