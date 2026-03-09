use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdin_bytes = match read_stdin_if_needed(&args) {
        Ok(stdin_bytes) => stdin_bytes,
        Err(error) => {
            eprintln!("failed to read stdin: {error}");
            return ExitCode::from(1);
        }
    };

    match tlsh_rs::cli::run_with_stdin(args, stdin_bytes.as_deref()) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn read_stdin_if_needed(args: &[String]) -> Result<Option<Vec<u8>>, std::io::Error> {
    if !contains_stdin_marker(args) {
        return Ok(None);
    }

    let mut buffer = Vec::new();
    match std::io::stdin().read_to_end(&mut buffer) {
        Ok(_) => {}
        Err(error) => return Err(error),
    }
    Ok(Some(buffer))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_is_a_thin_non_successful_wrapper_under_test_harness_args() {
        assert_ne!(main(), ExitCode::SUCCESS);
    }

    #[test]
    fn read_stdin_if_needed_skips_when_dash_is_absent() {
        assert_eq!(
            read_stdin_if_needed(&["hash".to_string(), "file.bin".to_string()]).unwrap(),
            None
        );
    }

    #[test]
    fn contains_stdin_marker_checks_for_dash_only() {
        assert!(!contains_stdin_marker(&[
            "hash".to_string(),
            "file.bin".to_string()
        ]));
        assert!(contains_stdin_marker(&[
            "hash".to_string(),
            "-".to_string()
        ]));
    }
}
