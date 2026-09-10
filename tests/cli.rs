//! End-to-end tests that run the compiled `haiku` binary as a subprocess.
//!
//! Each test points `XDG_DATA_HOME` at a fresh temp dir so `new`/`list`/
//! `random` never touch (or collide with) the real user data directory or
//! each other, since tests run concurrently.

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd(data_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("haiku").unwrap();
    cmd.env("XDG_DATA_HOME", data_dir);
    cmd
}

#[test]
fn check_accepts_a_valid_haiku_on_stdin() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .arg("check")
        .write_stdin("an old silent pond\na frog jumps into the pond\nsplash silence again\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("5-7-5 ok"));
}

#[test]
fn check_rejects_wrong_syllable_counts() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .arg("check")
        .write_stdin("too short\nstill not seventeen\nway way off\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("5-7-5"));
}

#[test]
fn check_reads_from_file() {
    let tmp = tempfile::tempdir().unwrap();
    let haiku_file = tmp.path().join("haiku.txt");
    std::fs::write(
        &haiku_file,
        "an old silent pond\na frog jumps into the pond\nsplash silence again\n",
    )
    .unwrap();

    cmd(tmp.path())
        .arg("check")
        .arg("--file")
        .arg(&haiku_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("5-7-5 ok"));
}

#[test]
fn check_reports_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .arg("check")
        .arg("--file")
        .arg(tmp.path().join("does-not-exist.txt"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("couldn't read"));
}

#[test]
fn list_on_empty_store_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("no haikus saved yet"));
}

#[test]
fn random_on_empty_store_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .arg("random")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no haikus saved yet"));
}

#[test]
fn new_dry_run_does_not_save() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .args([
            "new",
            "an old silent pond",
            "a frog jumps into the pond",
            "splash silence again",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("looks good"));

    cmd(tmp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("no haikus saved yet"));
}

#[test]
fn new_then_list_then_random_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .args([
            "new",
            "an old silent pond",
            "a frog jumps into the pond",
            "splash silence again",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("saved"));

    cmd(tmp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("(5-7-5)"));

    cmd(tmp.path())
        .arg("random")
        .assert()
        .success()
        .stdout(predicate::str::contains("silent pond"));
}

#[test]
fn new_rejects_wrong_syllable_counts_given_as_args() {
    // With no interactive terminal and no piped follow-up input, a rejected
    // line can't be corrected, so this surfaces as an EOF error rather than
    // the underlying `BadSyllables` error.
    let tmp = tempfile::tempdir().unwrap();
    cmd(tmp.path())
        .args(["new", "too short", "still not seventeen", "way way off"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(need 5)"))
        .stderr(predicate::str::contains("unexpected end of input"));
}
