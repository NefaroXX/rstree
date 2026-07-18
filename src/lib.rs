//! `ls-tree` core logic: recursively render a directory hierarchy using the
//! classic `tree` drawing characters (`├──`, `└──`, `│   `).
//!
//! Standard library only — no external dependencies. The rendering is written
//! against a generic [`std::io::Write`] so it can be tested without touching a
//! terminal or the filesystem.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Options controlling how the tree is rendered.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Show entries whose name starts with `.` (hidden on Unix). Defaults to
    /// `false`, matching the behaviour of the standard `tree` command.
    pub show_hidden: bool,
    /// Maximum directory depth to descend into. `None` means unlimited.
    pub max_depth: Option<usize>,
}

/// Counts produced while rendering the tree.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TreeStats {
    pub directories: u64,
    pub files: u64,
}

/// Render the directory tree rooted at `root` into `out`.
///
/// `root` must be a path to a directory. Returns [`TreeStats`] on success.
///
/// Root-level errors are surfaced as [`io::Error`] so callers can print a
/// friendly message (e.g. not found, not a directory). A root that exists but
/// cannot be read returns a minimal tree containing a single `[Access Denied]`
/// line and `TreeStats { directories: 1, files: 0 }`.
pub fn generate(root: &Path, out: &mut impl Write, opts: &Options) -> io::Result<TreeStats> {
    let name = root_display_name(root)?;

    match fs::metadata(root) {
        Ok(meta) if meta.is_dir() => {
            writeln!(out, "{}", name)?;
            let mut stats = TreeStats {
                directories: 1,
                files: 0,
            };
            walk(root, "", 0, opts, out, &mut stats)?;
            Ok(stats)
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        )),
        Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
            writeln!(out, "{}", name)?;
            writeln!(out, "└── [Access Denied]")?;
            Ok(TreeStats {
                directories: 1,
                files: 0,
            })
        }
        Err(e) => Err(e),
    }
}

/// Choose a friendly display name for the root (e.g. the real folder name when
/// the caller passed `.`).
fn root_display_name(root: &Path) -> io::Result<String> {
    if root.as_os_str().is_empty() || root.as_os_str() == "." {
        let cwd = std::env::current_dir()?;
        return Ok(cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string());
    }
    Ok(match root.file_name().and_then(|n| n.to_str()) {
        Some(s) => s.to_string(),
        None => root.to_string_lossy().into_owned(),
    })
}

/// Recursively print the contents of `dir`, indenting each level with the
/// classic tree characters. `depth` is the depth of `dir` relative to the root
/// (root = 0) and is used only for `--max-depth` limiting.
fn walk(
    dir: &Path,
    prefix: &str,
    depth: usize,
    opts: &Options,
    out: &mut impl Write,
    stats: &mut TreeStats,
) -> io::Result<()> {
    let mut entries = match fs::read_dir(dir) {
        Ok(rd) => match rd.collect::<Result<Vec<_>, io::Error>>() {
            Ok(entries) => entries,
            Err(e) => {
                // A read error part-way through (e.g. one unreadable entry)
                // should not abort the whole traversal.
                writeln!(out, "{}{}", prefix, error_label(&e))?;
                return Ok(());
            }
        },
        Err(e) => {
            writeln!(out, "{}{}", prefix, error_label(&e))?;
            return Ok(());
        }
    };

    if !opts.show_hidden {
        entries.retain(|e| !e.file_name().to_string_lossy().starts_with('.'));
    }

    // Locale-independent, deterministic ordering.
    entries.sort_by_key(|e| e.file_name());

    let last_index = entries.len().saturating_sub(1);
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == last_index;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name().to_string_lossy().into_owned();

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                writeln!(out, "{}{}{}  [Error: {}]", prefix, connector, name, e)?;
                stats.files += 1;
                continue;
            }
        };

        // Never descend into symlinks. This avoids infinite loops on symlink
        // cycles and keeps the output to what is directly visible. The link
        // target is shown for clarity.
        if file_type.is_symlink() {
            let target = fs::read_link(entry.path())
                .map(|t| t.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "<unknown>".to_string());
            writeln!(out, "{}{}{} -> {}", prefix, connector, name, target)?;
            stats.files += 1;
            continue;
        }

        if file_type.is_dir() {
            writeln!(out, "{}{}{}/", prefix, connector, name)?;
            stats.directories += 1;

            let descend = opts.max_depth.is_none_or(|max| depth + 1 < max);
            if descend {
                let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                walk(&entry.path(), &child_prefix, depth + 1, opts, out, stats)?;
            }
        } else {
            writeln!(out, "{}{}{}", prefix, connector, name)?;
            stats.files += 1;
        }
    }

    Ok(())
}

fn error_label(e: &io::Error) -> String {
    if e.kind() == io::ErrorKind::PermissionDenied {
        "[Access Denied]".to_string()
    } else {
        format!("[Error: {}]", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// A temporary directory that removes itself on drop (std-only; we avoid
    /// the `tempfile` crate to keep `ls-tree` dependency-free).
    struct Tmp(PathBuf);
    impl Tmp {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Create a uniquely-named temporary directory.
    fn make_tmp() -> Tmp {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("ls-tree-test-{}-{}", std::process::id(), n));
        fs::create_dir_all(&p).expect("create temp dir");
        Tmp(p)
    }

    fn render(dir: &Path, opts: &Options) -> (String, TreeStats) {
        let mut buf = Vec::new();
        let stats = generate(dir, &mut buf, opts).expect("generate should succeed");
        (String::from_utf8(buf).unwrap(), stats)
    }

    /// Build a small tree:
    /// ```text
    /// a/
    ///   a1.txt  a2.txt  sub/deep.txt
    /// b.txt
    /// .hidden_file
    /// .hidden_dir/x.txt
    /// ```
    fn scaffold() -> Tmp {
        let tmp = make_tmp();
        fs::create_dir(tmp.path().join("a")).unwrap();
        fs::write(tmp.path().join("a").join("a1.txt"), b"").unwrap();
        fs::write(tmp.path().join("a").join("a2.txt"), b"").unwrap();
        fs::create_dir(tmp.path().join("a").join("sub")).unwrap();
        fs::write(tmp.path().join("a").join("sub").join("deep.txt"), b"").unwrap();
        fs::write(tmp.path().join("b.txt"), b"").unwrap();
        fs::write(tmp.path().join(".hidden_file"), b"").unwrap();
        fs::create_dir(tmp.path().join(".hidden_dir")).unwrap();
        fs::write(tmp.path().join(".hidden_dir").join("x.txt"), b"").unwrap();
        tmp
    }

    #[test]
    fn renders_recursive_hierarchy() {
        let tmp = scaffold();
        let (out, stats) = render(tmp.path(), &Options::default());
        assert!(out.contains("a/"));
        assert!(out.contains("a1.txt"));
        assert!(out.contains("a2.txt"));
        assert!(out.contains("sub/"));
        assert!(out.contains("deep.txt"));
        assert!(out.contains("b.txt"));
        // Hidden entries are excluded by default.
        assert!(!out.contains(".hidden_file"));
        assert!(!out.contains(".hidden_dir"));
        assert_eq!(
            stats,
            TreeStats {
                directories: 3,
                files: 4
            }
        );
    }

    #[test]
    fn shows_hidden_with_flag() {
        let tmp = scaffold();
        let opts = Options {
            show_hidden: true,
            max_depth: None,
        };
        let (out, stats) = render(tmp.path(), &opts);
        assert!(out.contains(".hidden_file"));
        assert!(out.contains(".hidden_dir/"));
        assert_eq!(
            stats,
            TreeStats {
                directories: 4,
                files: 6
            }
        );
    }

    #[test]
    fn symlink_does_not_cause_cycle() {
        let tmp = scaffold();
        // A symlink pointing back at the tree root would loop forever if we
        // descended into symlinks.
        #[cfg(unix)]
        std::os::unix::fs::symlink(tmp.path(), tmp.path().join("loop")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(tmp.path(), tmp.path().join("loop")).unwrap();

        let (out, _stats) = render(tmp.path(), &Options::default());
        assert!(out.contains("loop"));
        assert!(out.contains("->"));
    }

    #[test]
    fn handles_permission_denied_gracefully() {
        let tmp = scaffold();
        let locked = tmp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(locked.join("secret.txt"), b"").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        }
        // If we can still read it (e.g. running as root) the assertion cannot
        // hold — skip rather than fail.
        if fs::read_dir(&locked).is_ok() {
            return;
        }

        let (out, _stats) = render(tmp.path(), &Options::default());
        assert!(out.contains("[Access Denied]"));

        // Restore permissions so the temp dir can be cleaned up.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }

    #[test]
    fn root_permission_denied_is_minimal() {
        let tmp = make_tmp();
        let locked = tmp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        }
        if fs::read_dir(&locked).is_ok() {
            return;
        }

        let mut buf = Vec::new();
        let stats = generate(&locked, &mut buf, &Options::default()).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("[Access Denied]"));
        assert_eq!(
            stats,
            TreeStats {
                directories: 1,
                files: 0
            }
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
}
