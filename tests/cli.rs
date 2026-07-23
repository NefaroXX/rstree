use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ls_tree_bin() -> Command {
    Command::new(
        std::env::var("CARGO_BIN_EXE_ls-tree")
            .expect("CARGO_BIN_EXE_ls-tree should be set by cargo for integration tests"),
    )
}

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

#[test]
fn json_flag_outputs_valid_json() {
    let tmp = make_tmp();
    fs::write(tmp.join("test.txt"), b"data").unwrap();

    let output = ls_tree_bin()
        .arg("--json")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    assert_eq!(parsed["type"], "directory");
    assert!(parsed["children"].is_array());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn size_flag_shows_sizes() {
    let tmp = make_tmp();
    fs::write(tmp.join("data.bin"), b"hello world").unwrap();

    let output = ls_tree_bin().arg("-s").current_dir(&tmp).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('['));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn dirs_only_flag() {
    let tmp = make_tmp();
    fs::create_dir(tmp.join("mydir")).unwrap();
    fs::write(tmp.join("file.txt"), b"").unwrap();

    let output = ls_tree_bin()
        .arg("--dirs-only")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("mydir"));
    assert!(!stdout.contains("file.txt"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn prune_flag_omits_empty_dirs() {
    let tmp = make_tmp();
    fs::create_dir(tmp.join("full")).unwrap();
    fs::write(tmp.join("full").join("f.txt"), b"").unwrap();
    fs::create_dir(tmp.join("empty")).unwrap();

    let output = ls_tree_bin()
        .arg("--prune")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("full/"));
    assert!(!stdout.contains("empty/"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn max_depth_limits_output() {
    let tmp = make_tmp();
    fs::create_dir_all(tmp.join("a").join("b").join("c")).unwrap();
    fs::write(tmp.join("a").join("b").join("c").join("deep.txt"), b"").unwrap();

    let output = ls_tree_bin()
        .arg("-L")
        .arg("2")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("a/"));
    assert!(stdout.contains("b/"));
    assert!(!stdout.contains("deep.txt"));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn sort_flag_does_not_crash() {
    let tmp = make_tmp();
    fs::write(tmp.join("z_file.txt"), b"").unwrap();
    fs::write(tmp.join("a_file.txt"), b"").unwrap();

    for sort_val in &["name", "size", "time"] {
        let output = ls_tree_bin()
            .arg("--sort")
            .arg(sort_val)
            .current_dir(&tmp)
            .output()
            .unwrap();
        assert!(output.status.success(), "sort={} should succeed", sort_val);
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn icons_flag_does_not_crash() {
    let tmp = make_tmp();
    fs::write(tmp.join("file.rs"), b"").unwrap();

    let output = ls_tree_bin()
        .arg("--icons")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(output.status.success());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn human_readable_flag_works() {
    let tmp = make_tmp();
    fs::write(tmp.join("big.bin"), b"x".repeat(2048)).unwrap();

    let output = ls_tree_bin().arg("-h").current_dir(&tmp).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('['));
    let _ = fs::remove_dir_all(&tmp);
}
