use std::fs;
use std::path::{Path, PathBuf};

use thefmt::format_markdown;

#[test]
fn formats_each_testcase() {
    for testcase_dir in testcase_dirs("tests/testcases") {
        let before = read_testcase_file(&testcase_dir, "before.md");
        let after = read_testcase_file(&testcase_dir, "after.md");

        let formatted = format_markdown(&before).unwrap();

        assert_eq!(formatted, after, "testcase: {}", testcase_dir.display());
    }
}

fn testcase_dirs(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut dirs = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn read_testcase_file(testcase_dir: &Path, name: &str) -> String {
    fs::read_to_string(testcase_dir.join(name)).unwrap()
}
