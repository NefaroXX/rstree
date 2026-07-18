//! End-to-end tests that exercise the compiled `ls-tree` binary the way a user
//! would run it.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ls_tree_bin() -> Command {
    Command::new(
        std::env::var("CARGO_BIN_EXE_ls-tree")
            .expect("CARGO_BIN_EXE_ls-tree should be set by cargo for integration tests"),
    )
}

/// Create a uniquely-named temp dir (std-only) and return its path. The caller
/// is responsible for removing it.
fn make_tmp() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static C: AtomicU64 = AtomicU64::new(0);
    let n = C.fetch_add(1, Ordering::SeqCst);
    let mut p = std::env::temp_dir();
    p.push(format!("ls-tree-cli-test-{}-{}", std::process::id(), n));
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn prints_tree_for_default_directory() {
    let tmp = make_tmp();
    fs::create_dir(tmp.join("sub")).unwrap();
    fs::write(tmp.join("sub").join("a.txt"), b"").unwrap();
    fs::write(tmp.join("top.txt"), b"").unwrap();

    let output = ls_tree_bin().current_dir(&tmp).output().unwrap();
    let _ = fs::remove_dir_all(&tmp);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sub/"));
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("top.txt"));
    assert!(stdout.contains("directories,"));
}

#[test]
fn reports_missing_path() {
    let output = ls_tree_bin()
        .arg("/path/that/does/not/exist/xyz")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("does not exist"));
}

#[test]
fn shows_hidden_with_all_flag() {
    let tmp = make_tmp();
    fs::write(tmp.join(".secret"), b"").unwrap();

    let output = ls_tree_bin().arg("-a").current_dir(&tmp).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(".secret"));

    let output2 = ls_tree_bin().current_dir(&tmp).output().unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();
    assert!(!stdout2.contains(".secret"));

    let _ = fs::remove_dir_all(&tmp);
}
