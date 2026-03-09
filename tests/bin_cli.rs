use std::io::Write;
use std::process::{Command, Stdio};

const SMALL_HASH: &str = "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";

#[test]
fn bin_hashes_file_successfully() {
    let output = tlsh_command()
        .arg("hash")
        .arg(fixture("small.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), SMALL_HASH);
    assert!(output.stderr.is_empty());
}

#[test]
fn bin_reports_parse_errors_on_stderr() {
    let output = tlsh_command().arg("wat").output().unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown command")
    );
}

#[test]
fn bin_hashes_stdin_when_dash_is_used() {
    let input = std::fs::read(fixture("small.txt")).unwrap();
    let mut child = tlsh_command()
        .arg("hash")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(&input).unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), SMALL_HASH);
    assert!(output.stderr.is_empty());
}

#[test]
fn bin_reports_cli_errors_from_execution() {
    let output = tlsh_command()
        .arg("hash")
        .arg("-")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("input too short for TLSH")
    );
}

#[cfg(unix)]
#[test]
fn bin_reports_stdin_read_errors() {
    let directory = std::fs::File::open(env!("CARGO_MANIFEST_DIR")).unwrap();
    let output = tlsh_command()
        .arg("hash")
        .arg("-")
        .stdin(Stdio::from(directory))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("failed to read stdin")
    );
}

#[test]
fn bin_succeeds_with_empty_output_when_xref_filters_everything() {
    let output = tlsh_command()
        .arg("xref")
        .arg("--threshold")
        .arg("0")
        .arg(fixture("small.txt"))
        .arg(fixture("small2.txt"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_tlsh")
}

fn tlsh_command() -> Command {
    let mut command = Command::new(bin_path());
    configure_child_coverage(&mut command);
    command
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
fn configure_child_coverage(command: &mut Command) {
    command.env_remove("LLVM_PROFILE_FILE");
}

#[cfg(not(all(target_os = "windows", target_arch = "aarch64")))]
fn configure_child_coverage(_command: &mut Command) {}

fn fixture(name: &str) -> String {
    format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}
