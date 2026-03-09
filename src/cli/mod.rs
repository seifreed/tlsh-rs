mod application;
mod args;
mod io;
mod model;
mod presentation;

use std::io::{Read, Write};
use std::process::ExitCode;

pub fn run(args: Vec<String>) -> Result<String, String> {
    run_with_stdin(args, None)
}

pub fn run_with_stdin(args: Vec<String>, stdin_bytes: Option<&[u8]>) -> Result<String, String> {
    let command = args::parse(args)?;
    let mut context = io::CliContext::new(stdin_bytes);
    let output = match application::execute(command, &mut context) {
        Ok(output) => output,
        Err(error) => return Err(error.to_string()),
    };
    Ok(presentation::render(output))
}

pub fn usage() -> String {
    args::usage()
}

pub fn run_with_io(
    args: Vec<String>,
    stdin: &mut impl Read,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> ExitCode {
    let stdin_buffer = match collect_stdin_if_needed(&args, stdin) {
        Ok(buffer) => buffer,
        Err(error) => {
            let _ = writeln!(stderr, "failed to read stdin: {error}");
            return ExitCode::from(1);
        }
    };

    match run_with_stdin(args, stdin_buffer.as_deref()) {
        Ok(output) => {
            if !output.is_empty() {
                let _ = writeln!(stdout, "{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            ExitCode::from(1)
        }
    }
}

fn collect_stdin_if_needed(
    args: &[String],
    stdin: &mut impl Read,
) -> Result<Option<Vec<u8>>, std::io::Error> {
    let mut needs_stdin = false;
    for arg in args {
        if arg == "-" {
            needs_stdin = true;
            break;
        }
    }
    if !needs_stdin {
        return Ok(None);
    }

    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;
    Ok(Some(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error};

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::other("boom"))
        }
    }

    fn fixture(name: &str) -> String {
        format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn run_hashes_a_file() {
        let output = run(vec!["hash".to_string(), fixture("small.txt")]).unwrap();
        assert!(output.starts_with("T1F8A0220"));
    }

    #[test]
    fn run_with_stdin_hashes_stdin() {
        let input = std::fs::read(fixture("small.txt")).unwrap();
        let output =
            run_with_stdin(vec!["hash".to_string(), "-".to_string()], Some(&input)).unwrap();
        assert!(output.starts_with("T1F8A0220"));
    }

    #[test]
    fn run_surfaces_parse_errors() {
        let error = run(vec!["unknown".to_string()]).unwrap_err();
        assert!(error.contains("unknown command"));
        assert_eq!(usage(), args::usage());
    }

    #[test]
    fn run_surfaces_execution_errors() {
        let error = run(vec!["hash".to_string(), "-".to_string()]).unwrap_err();
        assert!(error.contains("stdin was requested"));
    }

    #[test]
    fn collect_stdin_skips_when_dash_is_absent() {
        let mut stdin = Cursor::new(b"ignored".to_vec());
        let buffer =
            collect_stdin_if_needed(&["hash".to_string(), "file.bin".to_string()], &mut stdin)
                .unwrap();
        assert_eq!(buffer, None);
    }

    #[test]
    fn collect_stdin_reads_when_dash_is_present() {
        let mut stdin = Cursor::new(b"stdin".to_vec());
        let buffer =
            collect_stdin_if_needed(&["hash".to_string(), "-".to_string()], &mut stdin).unwrap();
        assert_eq!(buffer, Some(b"stdin".to_vec()));
    }

    #[test]
    fn run_with_io_writes_stdout_on_success() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run_with_io(
            vec!["hash".to_string(), fixture("small.txt")],
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(String::from_utf8(stdout).unwrap().contains("T1F8A0220"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn run_with_io_writes_stderr_on_cli_error() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run_with_io(
            vec!["hash".to_string()],
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, ExitCode::from(1));
        assert!(stdout.is_empty());
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("Usage: tlsh hash")
        );
    }

    #[test]
    fn run_with_io_reports_stdin_read_failure() {
        let mut stdin = FailingReader;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run_with_io(
            vec!["hash".to_string(), "-".to_string()],
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, ExitCode::from(1));
        assert!(stdout.is_empty());
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("failed to read stdin")
        );
    }

    #[test]
    fn run_with_io_skips_stdout_for_empty_output() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run_with_io(
            vec![
                "xref".to_string(),
                "--threshold".to_string(),
                "-1".to_string(),
                fixture("small.txt"),
                fixture("small2.txt"),
            ],
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
