//! Recursively render a directory hierarchy using the classic `tree` drawing
//! characters (`├──`, `└──`, `│   `) or output machine-readable JSON.
//! Supports colours, file sizes, `.gitignore` awareness, sorting, pruning, and
//! file-type icons. The render target is a generic [`std::io::Write`] so the
//! library can be tested without a terminal or the filesystem.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::Serialize;

/// Controls how the tree is gathered and rendered.
#[derive(Debug, Clone)]
pub struct Options {
    pub show_hidden: bool,
    pub max_depth: Option<usize>,
    pub json: bool,
    pub show_size: bool,
    pub human_readable: bool,
    pub color: bool,
    pub git_ignore: bool,
    pub dirs_only: bool,
    pub sort_by: SortField,
    pub prune: bool,
    pub show_icons: bool,
    pub show_total_size: bool,
}

impl Default for Options {
    fn default() -> Self {
        let color = std::io::IsTerminal::is_terminal(&std::io::stdout())
            && std::env::var("NO_COLOR").is_err();
        Self {
            show_hidden: false,
            max_depth: None,
            json: false,
            show_size: false,
            human_readable: false,
            color,
            git_ignore: false,
            dirs_only: false,
            sort_by: SortField::Name,
            prune: false,
            show_icons: false,
            show_total_size: false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    #[default]
    Name,
    Size,
    Time,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TreeStats {
    pub directories: u64,
    pub files: u64,
}

#[derive(Debug, Serialize)]
pub struct TreeNode {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeNode>,
}

pub fn generate(root: &Path, out: &mut impl Write, opts: &Options) -> io::Result<TreeStats> {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    let name = root_display_name(root)?;

    match fs::metadata(root) {
        Ok(meta) if meta.is_dir() => {
            let gi = if opts.git_ignore {
                build_gitignore(&canonical_root)
            } else {
                None
            };

            let root_node = match build_tree(root, &canonical_root, opts, 0, &gi) {
                Ok(node) => node,
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    writeln!(out, "{}", name)?;
                    writeln!(out, "└── [Access Denied]")?;
                    return Ok(TreeStats {
                        directories: 1,
                        files: 0,
                    });
                }
                Err(e) => return Err(e),
            };

            if opts.json {
                serde_json::to_writer_pretty(&mut *out, &root_node)?;
                writeln!(out)?;
                Ok(count_nodes(&root_node))
            } else {
                writeln!(out, "{}", name)?;
                let mut stats = TreeStats {
                    directories: 1,
                    files: 0,
                };
                render_children(&root_node.children, "", opts, out, &mut stats)?;
                Ok(stats)
            }
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

fn root_display_name(root: &Path) -> io::Result<String> {
    let empty = root.as_os_str().is_empty() || root.as_os_str() == ".";
    if empty {
        let cwd = std::env::current_dir()?;
        Ok(cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string())
    } else {
        Ok(match root.file_name().and_then(|n| n.to_str()) {
            Some(s) => s.to_string(),
            None => root.to_string_lossy().into_owned(),
        })
    }
}

fn build_tree(
    dir: &Path,
    root: &Path,
    opts: &Options,
    depth: usize,
    gitignore: &Option<Gitignore>,
) -> io::Result<TreeNode> {
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| dir.to_string_lossy().into_owned());

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied && depth > 0 => {
            return Ok(TreeNode {
                name: dir_name,
                size: None,
                entry_type: "directory".to_string(),
                target: None,
                children: vec![TreeNode {
                    name: "[Access Denied]".to_string(),
                    size: None,
                    entry_type: "file".to_string(),
                    target: None,
                    children: vec![],
                }],
            });
        }
        Err(e) => return Err(e),
    };
    let mut raw: Vec<fs::DirEntry> = read_dir.filter_map(|r| r.ok()).collect();

    if !opts.show_hidden {
        raw.retain(|e| !e.file_name().to_string_lossy().starts_with('.'));
    }

    if let Some(ref gi) = *gitignore {
        raw.retain(|e| {
            let path = e.path();
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            !is_ignored(&path, is_dir, root, gi)
        });
    }

    if opts.dirs_only {
        raw.retain(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false));
    }

    sort_entries(&mut raw, opts.sort_by);

    let mut children: Vec<TreeNode> = Vec::new();

    for entry in &raw {
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().into_owned();

        if ft.is_symlink() {
            let target = fs::read_link(entry.path())
                .map(|t| t.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "<unknown>".to_string());
            children.push(TreeNode {
                name: format!("{} -> {}", name, target),
                size: if opts.show_size {
                    Some(meta.len())
                } else {
                    None
                },
                entry_type: "symlink".to_string(),
                target: Some(target),
                children: vec![],
            });
        } else if ft.is_dir() {
            let descend = opts.max_depth.is_none_or(|max| depth + 1 < max);
            if descend {
                let child = build_tree(&entry.path(), root, opts, depth + 1, gitignore)?;
                if opts.prune && child.children.is_empty() {
                    continue;
                }
                let child_size = child.size.unwrap_or(0);
                children.push(TreeNode {
                    name,
                    size: if opts.show_size || opts.show_total_size {
                        Some(child_size)
                    } else {
                        None
                    },
                    entry_type: "directory".to_string(),
                    target: None,
                    children: child.children,
                });
            } else {
                children.push(TreeNode {
                    name,
                    size: if opts.show_size {
                        Some(meta.len())
                    } else {
                        None
                    },
                    entry_type: "directory".to_string(),
                    target: None,
                    children: vec![],
                });
            }
        } else {
            children.push(TreeNode {
                name,
                size: if opts.show_size {
                    Some(meta.len())
                } else {
                    None
                },
                entry_type: "file".to_string(),
                target: None,
                children: vec![],
            });
        }
    }

    let total_size: u64 = children.iter().filter_map(|c| c.size).sum();

    Ok(TreeNode {
        name: dir_name,
        size: if opts.show_total_size || (opts.show_size && !children.is_empty()) {
            Some(total_size)
        } else {
            None
        },
        entry_type: "directory".to_string(),
        target: None,
        children,
    })
}

fn count_nodes(node: &TreeNode) -> TreeStats {
    let mut dirs = 0u64;
    let mut files = 0u64;
    if node.entry_type == "directory" {
        dirs += 1;
    } else {
        files += 1;
    }
    for child in &node.children {
        let s = count_nodes(child);
        dirs += s.directories;
        files += s.files;
    }
    TreeStats {
        directories: dirs,
        files,
    }
}

fn render_children(
    children: &[TreeNode],
    prefix: &str,
    opts: &Options,
    out: &mut impl Write,
    stats: &mut TreeStats,
) -> io::Result<()> {
    let last = children.len().saturating_sub(1);
    for (i, child) in children.iter().enumerate() {
        let is_last = i == last;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        let display = format_entry(child, opts);

        match child.entry_type.as_str() {
            "directory" => {
                writeln!(out, "{}{}{}/", prefix, connector, display)?;
                stats.directories += 1;
                render_children(&child.children, &child_prefix, opts, out, &mut *stats)?;
            }
            "symlink" => {
                writeln!(out, "{}{}{}", prefix, connector, display)?;
                stats.files += 1;
            }
            _ => {
                writeln!(out, "{}{}{}", prefix, connector, display)?;
                stats.files += 1;
            }
        }
    }
    Ok(())
}

fn format_entry(node: &TreeNode, opts: &Options) -> String {
    let icon = if opts.show_icons {
        file_icon(&node.name, &node.entry_type, false)
    } else {
        ""
    };

    let display_name = if node.entry_type == "symlink" {
        if let Some(ref target) = node.target {
            let bare = node.name.trim_end_matches(&format!(" -> {}", target));
            let coloured = colorize(bare, "symlink", opts.color);
            format!("{} -> {}", coloured, target)
        } else {
            colorize(&node.name, "symlink", opts.color)
        }
    } else {
        colorize(&node.name, &node.entry_type, opts.color)
    };

    let suffix = if opts.human_readable {
        if let Some(sz) = node.size {
            format!(" [{}]", format_size_human(sz))
        } else {
            String::new()
        }
    } else if opts.show_size {
        if let Some(sz) = node.size {
            format!(" [{}]", sz)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!("{}{}{}", icon, display_name, suffix)
}

fn format_size_human(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut s = size as f64;
    let mut idx = 0;
    while s >= 1024.0 && idx < UNITS.len() - 1 {
        s /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{}", size)
    } else if s < 10.0 {
        format!("{:.1}{}", s, UNITS[idx])
    } else {
        format!("{:.0}{}", s, UNITS[idx])
    }
}

mod colour {
    pub const RESET: &str = "\x1b[0m";
    pub const DIR: &str = "\x1b[1;34m";
    pub const EXE: &str = "\x1b[32m";
    pub const SYMLINK: &str = "\x1b[36m";
}

fn colorize(name: &str, entry_type: &str, enabled: bool) -> String {
    if !enabled {
        return name.to_string();
    }
    let code = match entry_type {
        "directory" => colour::DIR,
        "symlink" => colour::SYMLINK,
        _ => colour::EXE,
    };
    format!("{}{}{}", code, name, colour::RESET)
}

fn file_icon(name: &str, entry_type: &str, _is_exe: bool) -> &'static str {
    match entry_type {
        "directory" => "\u{1f4c1} ",
        "symlink" => "\u{1f517} ",
        _ => {
            if name.ends_with(".rs") {
                "\u{1f980} "
            } else if name.ends_with(".md") {
                "\u{1f4dd} "
            } else if name.ends_with(".toml") || name.ends_with(".json") || name.ends_with(".yml") {
                "\u{2699}\u{fe0f}  "
            } else if name.ends_with(".png")
                || name.ends_with(".jpg")
                || name.ends_with(".svg")
                || name.ends_with(".ico")
            {
                "\u{1f5bc}\u{fe0f}  "
            } else {
                "\u{1f4c4} "
            }
        }
    }
}

fn sort_entries(entries: &mut [fs::DirEntry], field: SortField) {
    match field {
        SortField::Name => {
            entries.sort_by_key(|a| a.file_name());
        }
        SortField::Size => {
            entries.sort_by(|a, b| {
                let sa = a.metadata().ok().map(|m| m.len()).unwrap_or(0);
                let sb = b.metadata().ok().map(|m| m.len()).unwrap_or(0);
                sa.cmp(&sb)
            });
        }
        SortField::Time => {
            entries.sort_by(|a, b| {
                let ta = a
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::UNIX_EPOCH);
                let tb = b
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::UNIX_EPOCH);
                ta.cmp(&tb)
            });
        }
    }
}

fn build_gitignore(root: &Path) -> Option<Gitignore> {
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() || !gitignore_path.is_file() {
        return None;
    }
    let mut builder = GitignoreBuilder::new(root);
    builder.add(gitignore_path);
    builder.build().ok()
}

fn is_ignored(path: &Path, is_dir: bool, root: &Path, gi: &Gitignore) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    gi.matched(rel, is_dir).is_ignore()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

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

    fn scaffold() -> Tmp {
        let tmp = make_tmp();
        fs::create_dir(tmp.path().join("a")).unwrap();
        fs::write(tmp.path().join("a").join("a1.txt"), b"hello").unwrap();
        fs::write(tmp.path().join("a").join("a2.txt"), b"world").unwrap();
        fs::create_dir(tmp.path().join("a").join("sub")).unwrap();
        fs::write(tmp.path().join("a").join("sub").join("deep.txt"), b"deep").unwrap();
        fs::write(tmp.path().join("b.txt"), b"root").unwrap();
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
            ..Default::default()
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
        #[cfg(unix)]
        std::os::unix::fs::symlink(tmp.path(), tmp.path().join("loop")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(tmp.path(), tmp.path().join("loop")).unwrap();

        let (out, _) = render(tmp.path(), &Options::default());
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
        if fs::read_dir(&locked).is_ok() {
            return;
        }

        let (out, _) = render(tmp.path(), &Options::default());
        assert!(out.contains("[Access Denied]"));

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

    #[test]
    fn json_output_is_valid() {
        let tmp = scaffold();
        let opts = Options {
            json: true,
            ..Default::default()
        };
        let (out, stats) = render(tmp.path(), &opts);
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(parsed.get("name").is_some());
        assert_eq!(parsed["type"], "directory");
        assert!(stats.directories >= 1);
        assert!(parsed["children"].is_array());
    }

    #[test]
    fn json_output_serializes_symlinks() {
        let tmp = make_tmp();
        fs::write(tmp.path().join("target.txt"), b"hi").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("target.txt", tmp.path().join("link")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file("target.txt", tmp.path().join("link")).unwrap();

        let opts = Options {
            json: true,
            ..Default::default()
        };
        let (out, _) = render(tmp.path(), &opts);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let children = parsed["children"].as_array().unwrap();
        if children.len() == 2 {
            let has_symlink = children.iter().any(|c| c["type"] == "symlink");
            assert!(has_symlink, "output should contain a symlink: {}", out);
        }
    }

    #[test]
    fn size_flag_adds_size_info() {
        let tmp = scaffold();
        let opts = Options {
            show_size: true,
            ..Default::default()
        };
        let (out, _) = render(tmp.path(), &opts);
        assert!(out.contains('['));
    }

    #[test]
    fn dirs_only_shows_only_directories() {
        let tmp = scaffold();
        let opts = Options {
            dirs_only: true,
            ..Default::default()
        };
        let (out, stats) = render(tmp.path(), &opts);
        assert!(out.contains("a/"));
        assert!(out.contains("sub/"));
        assert!(!out.contains("a1.txt"));
        assert_eq!(stats.files, 0);
        assert!(stats.directories >= 2);
    }

    #[test]
    fn prune_removes_empty_dirs() {
        let tmp = make_tmp();
        fs::create_dir(tmp.path().join("full")).unwrap();
        fs::write(tmp.path().join("full").join("f.txt"), b"").unwrap();
        fs::create_dir(tmp.path().join("empty")).unwrap();

        let opts = Options {
            prune: true,
            ..Default::default()
        };
        let (out, _) = render(tmp.path(), &opts);
        assert!(out.contains("full/"));
        assert!(!out.contains("empty/"));
    }

    #[test]
    fn max_depth_limits_recursion() {
        let tmp = scaffold();
        let opts = Options {
            max_depth: Some(1),
            ..Default::default()
        };
        let (out, _) = render(tmp.path(), &opts);
        assert!(out.contains("a/"));
        assert!(out.contains("b.txt"));
        assert!(!out.contains("deep.txt"));
    }

    #[test]
    fn icons_are_rendered() {
        let tmp = scaffold();
        let opts = Options {
            show_icons: true,
            ..Default::default()
        };
        let (out, _) = render(tmp.path(), &opts);
        assert!(out.contains('\u{1f4c1}'));
    }
}
