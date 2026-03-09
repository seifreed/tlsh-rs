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

#[allow(clippy::question_mark)]
pub fn run_with_stdin(args: Vec<String>, stdin_bytes: Option<&[u8]>) -> Result<String, String> {
    let command = match parse_command(args) {
        Ok(command) => command,
        Err(error) => return Err(error),
    };
    let mut context = io::CliContext::new(stdin_bytes);
    execute_and_render(command, &mut context)
}

pub fn usage() -> String {
    args::usage()
}

pub fn run_with_io(
    args: Vec<String>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitCode {
    let stdin_buffer = match prepare_stdin_buffer(&args, stdin, stderr) {
        Some(buffer) => buffer,
        None => return ExitCode::from(1),
    };

    let result = run_with_stdin(args, stdin_buffer.as_deref());
    write_run_result(result, stdout, stderr)
}

fn parse_command(args: Vec<String>) -> Result<model::Command, String> {
    args::parse(args)
}

fn execute_and_render(
    command: model::Command,
    context: &mut io::CliContext<'_>,
) -> Result<String, String> {
    let output = match application::execute(command, context) {
        Ok(output) => output,
        Err(error) => return Err(error.to_string()),
    };
    Ok(presentation::render(output))
}

fn collect_stdin_if_needed(
    args: &[String],
    stdin: &mut dyn Read,
) -> Result<Option<Vec<u8>>, std::io::Error> {
    if !contains_stdin_marker(args) {
        return Ok(None);
    }

    read_stdin_buffer(stdin)
}

fn read_stdin_buffer(stdin: &mut dyn Read) -> Result<Option<Vec<u8>>, std::io::Error> {
    let mut buffer = Vec::new();
    match stdin.read_to_end(&mut buffer) {
        Ok(_) => Ok(Some(buffer)),
        Err(error) => Err(error),
    }
}

fn contains_stdin_marker(args: &[String]) -> bool {
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == "-" {
            return true;
        }
        index += 1;
    }
    false
}

fn write_stdin_error(stderr: &mut dyn Write, error: std::io::Error) -> ExitCode {
    let _ = writeln!(stderr, "failed to read stdin: {error}");
    ExitCode::from(1)
}

fn prepare_stdin_buffer(
    args: &[String],
    stdin: &mut dyn Read,
    stderr: &mut dyn Write,
) -> Option<Option<Vec<u8>>> {
    match collect_stdin_if_needed(args, stdin) {
        Ok(buffer) => Some(buffer),
        Err(error) => {
            let _ = write_stdin_error(stderr, error);
            None
        }
    }
}

fn write_run_result(
    result: Result<String, String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitCode {
    match result {
        Ok(output) => write_success_output(stdout, output),
        Err(message) => write_cli_error(stderr, &message),
    }
}

fn write_success_output(stdout: &mut dyn Write, output: String) -> ExitCode {
    if output.is_empty() {
        return ExitCode::SUCCESS;
    }
    let _ = writeln!(stdout, "{output}");
    ExitCode::SUCCESS
}

fn write_cli_error(stderr: &mut dyn Write, message: &str) -> ExitCode {
    let _ = writeln!(stderr, "{message}");
    ExitCode::from(1)
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
    fn parse_command_delegates_to_argument_parser() {
        let command = parse_command(vec!["hash".to_string(), fixture("small.txt")]).unwrap();
        let expected = args::parse(vec!["hash".to_string(), fixture("small.txt")]).unwrap();
        assert_eq!(command, expected);
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
    fn read_and_prepare_stdin_buffer_cover_success_and_error_paths() {
        let mut stdin = Cursor::new(b"stdin".to_vec());
        assert_eq!(
            read_stdin_buffer(&mut stdin).unwrap(),
            Some(b"stdin".to_vec())
        );

        let mut stdin = Cursor::new(b"stdin".to_vec());
        let mut stderr = Vec::new();
        let prepared = prepare_stdin_buffer(
            &["hash".to_string(), "-".to_string()],
            &mut stdin,
            &mut stderr,
        );
        assert_eq!(prepared, Some(Some(b"stdin".to_vec())));
        assert!(stderr.is_empty());

        let mut stdin = FailingReader;
        let mut stderr = Vec::new();
        let prepared = prepare_stdin_buffer(
            &["hash".to_string(), "-".to_string()],
            &mut stdin,
            &mut stderr,
        );
        assert_eq!(prepared, None);
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("failed to read stdin")
        );
    }

    #[test]
    fn contains_stdin_marker_detects_only_dash_arguments() {
        assert!(!contains_stdin_marker(&[
            "hash".to_string(),
            "file.bin".to_string()
        ]));
        assert!(contains_stdin_marker(&[
            "hash".to_string(),
            "-".to_string(),
            "file.bin".to_string()
        ]));
    }

    #[test]
    fn execute_and_render_returns_rendered_output() {
        let command = args::parse(vec!["hash".to_string(), fixture("small.txt")]).unwrap();
        let mut context = io::CliContext::new(None);
        let output = execute_and_render(command, &mut context).unwrap();
        assert!(output.starts_with("T1F8A0220"));
    }

    #[test]
    fn execute_and_render_stringifies_application_errors() {
        let command = args::parse(vec!["hash".to_string(), "-".to_string()]).unwrap();
        let mut context = io::CliContext::new(None);
        let error = execute_and_render(command, &mut context).unwrap_err();
        assert!(error.contains("stdin was requested"));
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
    fn write_helpers_cover_success_and_error_paths() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        assert_eq!(
            write_run_result(Ok("hello".to_string()), &mut stdout, &mut stderr),
            ExitCode::SUCCESS
        );
        assert_eq!(String::from_utf8(stdout).unwrap(), "hello\n");
        assert!(stderr.is_empty());

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            write_run_result(Ok(String::new()), &mut stdout, &mut stderr),
            ExitCode::SUCCESS
        );
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            write_run_result(Err("boom".to_string()), &mut stdout, &mut stderr),
            ExitCode::from(1)
        );
        assert!(stdout.is_empty());
        assert_eq!(String::from_utf8(stderr).unwrap(), "boom\n");
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
