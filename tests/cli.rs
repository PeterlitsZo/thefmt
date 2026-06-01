use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn formats_single_markdown_file_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("foobar.md");
    fs::write(&path, "# Hi").unwrap();

    Command::cargo_bin("thefmt")
        .unwrap()
        .arg(&path)
        .assert()
        .success();

    let formatted = fs::read_to_string(&path).unwrap();
    assert_eq!(formatted, "# Hi\n");
}

#[test]
fn rejects_missing_path() {
    Command::cargo_bin("thefmt")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: thefmt <file.md>"));
}

#[test]
fn rejects_extra_paths() {
    Command::cargo_bin("thefmt")
        .unwrap()
        .args(["one.md", "two.md"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: thefmt <file.md>"));
}

#[test]
fn reports_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.md");

    Command::cargo_bin("thefmt")
        .unwrap()
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read"));
}
