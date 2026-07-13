# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
