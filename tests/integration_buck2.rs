#[cfg(test)]
use assert_cmd::Command;

#[cfg(target_family = "unix")]
#[test]
fn test_buck2_latest() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("hashbang").unwrap();
    cmd.env("HASHBANG_CACHE", tmpdir.path().as_os_str());
    cmd.env("HASHBANG_CONFIG_URL", "file:./tests/data/buck2-url.toml");
    cmd.arg("--version");
    let assert = cmd.assert();
    let stdout = String::from_utf8(assert.get_output().stdout.to_vec()).unwrap();
    assert!(stdout.starts_with("buck2 "), "found {}", stdout);
    assert.success();
}

#[cfg(target_family = "unix")]
#[test]
fn test_buck2_fail() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("hashbang").unwrap();
    cmd.env("HASHBANG_CACHE", tmpdir.path().as_os_str());
    cmd.env("HASHBANG_CONFIG", "file:./tests/data/buck2-url.toml");
    cmd.arg("--totally-unknown-argument");
    let assert = cmd.assert();
    assert.failure();
}
