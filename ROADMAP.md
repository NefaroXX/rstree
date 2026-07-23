# Roadmap

## Done (v0.1.0)

- [x] Classic tree output with `├──`, `└──`, `│   ` connectors
- [x] JSON output (`--json`) — machine-readable for AI shells and OS tooling
- [x] File sizes (`-s`/`--size`) with human-readable formatting (`-h`)
- [x] ANSI colour support (directories, executables, symlinks; respects `NO_COLOR`)
- [x] `.gitignore` awareness (`--git-ignore`)
- [x] Sorting (`--sort=name/size/time`)
- [x] Directory-only mode (`--dirs-only`)
- [x] Prune empty directories (`--prune`)
- [x] Unicode file-type icons (`--icons`)
- [x] Recursive directory total sizes (`--total-size`)
- [x] Max-depth limiting (`-L`/`--max-depth`)
- [x] Hidden-file toggle (`-a`/`--all`)
- [x] Symlink-safe traversal (displayed but never descended into)
- [x] Graceful PermissionDenied handling (prints `[Access Denied]` inline)
- [x] Robust error handling with exit code discipline

## Current (v0.2.0 — in progress)

- _(none — planning next phase)_

## Next (v0.3.0 — library improvements)

- **Flexible output formatters** — decouple tree traversal from rendering via an `OutputFormatter` trait. Support tree, flat (find-style), and JSON formatters. Consumers of the library can register custom formatters without touching core logic.
- **Streaming / iterator API** — replace the all-in-memory `build_tree()` with a lazy `WalkIterator` that yields entries on demand. This enables processing of very large directories without holding the full tree in memory. The tree formatter will maintain a depth-stack internally rather than requiring a complete `TreeNode` tree.
- **Configuration file support** — read defaults from `.ls-tree.toml` in the current directory, plus a user-level config at `$XDG_CONFIG_HOME/ls-tree/config.toml` (or `~/.config/ls-tree/config.toml`). CLI flags override config values, which override built-in defaults.

## Future

- **Rust OS integration** — `no_std`-compatible core traversal (where the OS provides a minimal `Vfs` trait rather than `std::fs`). Conditional compilation for kernel-space builds.
- **AI shell integration** — structured output modes designed for LLM consumption: schema-constrained JSON variants, streaming-friendly delimited formats, and a `--watch` mode that emits diffs.
- **Cross-platform permission model** — abstract over Unix `mode` bits, Windows ACLs, and custom OS capability models via a `PermissionChecker` trait.
