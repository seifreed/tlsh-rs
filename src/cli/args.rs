use crate::TlshProfile;

use super::model::{
    Command, CompareOutputFormat, DiffCommand, HashCommand, HashManyCommand, HashOutputFormat,
    XrefCommand,
};

pub fn parse(args: Vec<String>) -> Result<Command, String> {
    if args.is_empty() {
        return Err(usage());
    }

    let command = args[0].as_str();
    let rest = &args[1..];

    if is_root_help_flag(command) {
        return Err(usage());
    }

    dispatch_command(command, rest)
}

pub fn usage() -> String {
    [
        "Usage:",
        "  tlsh hash [--profile 128-1|128-3|256-1|256-3] [--raw] [--format text|json] <file|->",
        "  tlsh hash-many [--profile 128-1|128-3|256-1|256-3] [--raw] [--format text|json] <file|-> <file|-> ...",
        "  tlsh diff [--profile 128-1|128-3|256-1|256-3] [--no-length] [--format text|json|sarif] <left> <right>",
        "  tlsh xref [--profile 128-1|128-3|256-1|256-3] [--no-length] [--format text|json|sarif] [--threshold N] <input> <input> ...",
        "",
        "Use '-' to read one binary input from stdin.",
        "For `diff` and `xref`, each input may be a file path, '-', or a TLSH digest string.",
    ]
    .join("\n")
}

fn hash_usage() -> String {
    "Usage: tlsh hash [--profile 128-1|128-3|256-1|256-3] [--raw] [--format text|json] <file|->"
        .to_string()
}

fn hash_many_usage() -> String {
    "Usage: tlsh hash-many [--profile 128-1|128-3|256-1|256-3] [--raw] [--format text|json] <file|-> <file|-> ..."
        .to_string()
}

fn diff_usage() -> String {
    "Usage: tlsh diff [--profile 128-1|128-3|256-1|256-3] [--no-length] [--format text|json|sarif] <left> <right>"
        .to_string()
}

fn xref_usage() -> String {
    "Usage: tlsh xref [--profile 128-1|128-3|256-1|256-3] [--no-length] [--format text|json|sarif] [--threshold N] <input> <input> ..."
        .to_string()
}

#[allow(clippy::question_mark, clippy::while_let_loop)]
fn parse_hash(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut raw = false;
    let mut format = HashOutputFormat::Text;
    let mut input = None::<String>;
    let mut index = 0usize;

    while index < args.len() {
        let arg = args[index].as_str();
        index += 1;

        if arg == "--profile" {
            profile = take_hash_profile(args, &mut index)?;
        } else if arg == "--raw" {
            raw = true;
        } else if arg == "--format" {
            format = take_hash_format(args, &mut index)?;
        } else if is_help_flag(arg) {
            return Err(hash_usage());
        } else if arg.starts_with("--") {
            return Err(unknown_option("hash", arg, hash_usage()));
        } else if input.is_some() {
            return Err(unexpected_extra_argument(arg, hash_usage()));
        } else {
            input = Some(arg.to_string());
        }
    }

    Ok(Command::Hash(HashCommand {
        profile,
        raw,
        format,
        input: match input {
            Some(input) => input,
            None => return Err(hash_usage()),
        },
    }))
}

#[allow(clippy::while_let_loop)]
fn parse_hash_many(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut raw = false;
    let mut format = HashOutputFormat::Text;
    let mut inputs = Vec::new();
    let mut parser = ArgCursor::new(args);

    loop {
        let arg = match parser.next() {
            Some(arg) => arg,
            None => break,
        };

        if arg == "--profile" {
            let value = parser.require_value("--profile")?;
            profile = parse_profile(value)?;
        } else if arg == "--raw" {
            raw = true;
        } else if arg == "--format" {
            let value = parser.require_value("--format")?;
            format = parse_hash_output_format(value)?;
        } else if is_help_flag(arg) {
            return Err(hash_many_usage());
        } else if arg.starts_with("--") {
            return Err(unknown_option("hash-many", arg, hash_many_usage()));
        } else {
            inputs.push(arg.to_string());
        }
    }

    if inputs.is_empty() {
        return Err(hash_many_usage());
    }

    Ok(Command::HashMany(HashManyCommand {
        profile,
        raw,
        format,
        inputs,
    }))
}

fn parse_diff(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut include_length = true;
    let mut format = CompareOutputFormat::Text;
    let mut values = Vec::with_capacity(2);
    let mut parser = ArgCursor::new(args);

    while let Some(arg) = parser.next() {
        if arg == "--profile" {
            profile = parse_profile_option(&mut parser)?;
        } else if arg == "--no-length" {
            include_length = false;
        } else if arg == "--format" {
            format = parse_compare_format_option(&mut parser)?;
        } else if is_help_flag(arg) {
            return Err(diff_usage());
        } else if arg.starts_with("--") {
            return Err(unknown_option("diff", arg, diff_usage()));
        } else {
            values.push(arg.to_string());
        }
    }

    if values.len() != 2 {
        return Err(diff_usage());
    }

    Ok(Command::Diff(DiffCommand {
        profile,
        include_length,
        format,
        left: values.remove(0),
        right: values.remove(0),
    }))
}

fn parse_xref(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut include_length = true;
    let mut format = CompareOutputFormat::Text;
    let mut threshold = None::<i32>;
    let mut inputs = Vec::new();
    let mut parser = ArgCursor::new(args);

    while let Some(arg) = parser.next() {
        if arg == "--profile" {
            profile = parse_profile_option(&mut parser)?;
        } else if arg == "--no-length" {
            include_length = false;
        } else if arg == "--format" {
            format = parse_compare_format_option(&mut parser)?;
        } else if arg == "--threshold" {
            threshold = Some(parse_threshold_option(&mut parser)?);
        } else if is_help_flag(arg) {
            return Err(xref_usage());
        } else if arg.starts_with("--") {
            return Err(unknown_option("xref", arg, xref_usage()));
        } else {
            inputs.push(arg.to_string());
        }
    }

    if inputs.len() < 2 {
        return Err(xref_usage());
    }

    Ok(Command::Xref(XrefCommand {
        profile,
        include_length,
        format,
        threshold,
        inputs,
    }))
}

fn parse_profile(value: &str) -> Result<TlshProfile, String> {
    match TlshProfile::from_cli_name(value) {
        Some(profile) => Ok(profile),
        None => Err(format!("unsupported profile: {value}")),
    }
}

fn is_help_flag(arg: &str) -> bool {
    arg == "--help" || arg == "-h"
}

fn parse_hash_output_format(value: &str) -> Result<HashOutputFormat, String> {
    match HashOutputFormat::from_cli_name(value) {
        Some(format) => Ok(format),
        None => Err(format!("unsupported format: {value}")),
    }
}

fn parse_compare_output_format(value: &str) -> Result<CompareOutputFormat, String> {
    match CompareOutputFormat::from_cli_name(value) {
        Some(format) => Ok(format),
        None => Err(format!("unsupported format: {value}")),
    }
}

fn parse_threshold(value: &str) -> Result<i32, String> {
    value
        .parse::<i32>()
        .map_err(|_| format!("invalid threshold: {value}"))
}

fn dispatch_command(command: &str, rest: &[String]) -> Result<Command, String> {
    match command {
        "hash" => parse_hash(rest),
        "hash-many" => parse_hash_many(rest),
        "diff" => parse_diff(rest),
        "xref" => parse_xref(rest),
        _ => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

fn is_root_help_flag(command: &str) -> bool {
    command == "--help" || command == "-h" || command == "help"
}

fn parse_profile_option(parser: &mut ArgCursor<'_>) -> Result<TlshProfile, String> {
    let value = parser.require_value("--profile")?;
    parse_profile(value)
}

fn parse_compare_format_option(parser: &mut ArgCursor<'_>) -> Result<CompareOutputFormat, String> {
    let value = parser.require_value("--format")?;
    parse_compare_output_format(value)
}

fn parse_threshold_option(parser: &mut ArgCursor<'_>) -> Result<i32, String> {
    let value = parser.require_value("--threshold")?;
    parse_threshold(value)
}

fn require_slice_value<'a>(
    args: &'a [String],
    index: &mut usize,
    option: &str,
) -> Result<&'a str, String> {
    if *index >= args.len() {
        return Err(format!("missing value for {option}"));
    }
    let value = args[*index].as_str();
    *index += 1;
    Ok(value)
}

fn take_hash_profile(args: &[String], index: &mut usize) -> Result<TlshProfile, String> {
    let value = require_slice_value(args, index, "--profile")?;
    parse_profile(value)
}

fn take_hash_format(args: &[String], index: &mut usize) -> Result<HashOutputFormat, String> {
    let value = require_slice_value(args, index, "--format")?;
    parse_hash_output_format(value)
}

fn unknown_option(command: &str, value: &str, usage: String) -> String {
    format!("unknown option for {command}: {value}\n\n{usage}")
}

fn unexpected_extra_argument(value: &str, usage: String) -> String {
    format!("unexpected extra argument: {value}\n\n{usage}")
}

struct ArgCursor<'a> {
    args: &'a [String],
    index: usize,
}

impl<'a> ArgCursor<'a> {
    fn new(args: &'a [String]) -> Self {
        Self { args, index: 0 }
    }

    fn next(&mut self) -> Option<&'a str> {
        let value = self.args.get(self.index)?;
        self.index += 1;
        Some(value.as_str())
    }

    fn require_value(&mut self, option: &str) -> Result<&'a str, String> {
        match self.next() {
            Some(value) => Ok(value),
            None => Err(format!("missing value for {option}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::model::{Command, CompareOutputFormat, HashOutputFormat};

    #[test]
    fn parse_hash_command_with_options() {
        let args = vec![
            "hash".to_string(),
            "--profile".to_string(),
            "256-3".to_string(),
            "--raw".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "sample.bin".to_string(),
        ];
        let command = parse(args).unwrap();
        assert_eq!(
            command,
            Command::Hash(HashCommand {
                profile: TlshProfile::full_256_3(),
                raw: true,
                format: HashOutputFormat::Json,
                input: "sample.bin".to_string(),
            })
        );
    }

    #[test]
    fn parse_diff_command_with_length_toggle() {
        let args = vec![
            "diff".to_string(),
            "--no-length".to_string(),
            "--format".to_string(),
            "sarif".to_string(),
            "left".to_string(),
            "right".to_string(),
        ];
        let command = parse(args).unwrap();
        assert_eq!(
            command,
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                format: CompareOutputFormat::Sarif,
                left: "left".to_string(),
                right: "right".to_string(),
            })
        );
    }

    #[test]
    fn parse_rejects_unknown_command() {
        let error = parse(vec!["wat".to_string()]).unwrap_err();
        assert!(error.contains("unknown command: wat"));
    }

    #[test]
    fn parse_rejects_missing_option_value() {
        let error = parse(vec!["hash".to_string(), "--profile".to_string()]).unwrap_err();
        assert_eq!(error, "missing value for --profile");
        let error = parse(vec!["hash".to_string(), "--format".to_string()]).unwrap_err();
        assert_eq!(error, "missing value for --format");
    }

    #[test]
    fn parse_rejects_bad_threshold() {
        let error = parse(vec![
            "xref".to_string(),
            "--threshold".to_string(),
            "NaN".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
        .unwrap_err();
        assert_eq!(error, "invalid threshold: NaN");
    }

    #[test]
    fn parse_hash_many_and_xref_commands() {
        let hash_many = parse(vec![
            "hash-many".to_string(),
            "--raw".to_string(),
            "a.bin".to_string(),
            "b.bin".to_string(),
        ])
        .unwrap();
        assert_eq!(
            hash_many,
            Command::HashMany(HashManyCommand {
                profile: TlshProfile::standard_t1(),
                raw: true,
                format: HashOutputFormat::Text,
                inputs: vec!["a.bin".to_string(), "b.bin".to_string()],
            })
        );

        let xref = parse(vec![
            "xref".to_string(),
            "--profile".to_string(),
            "256-1".to_string(),
            "--threshold".to_string(),
            "42".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
        .unwrap();
        assert_eq!(
            xref,
            Command::Xref(XrefCommand {
                profile: TlshProfile::full_256_1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                threshold: Some(42),
                inputs: vec!["left".to_string(), "right".to_string()],
            })
        );

        let xref_no_length = parse(vec![
            "xref".to_string(),
            "--no-length".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
        .unwrap();
        assert_eq!(
            xref_no_length,
            Command::Xref(XrefCommand {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                format: CompareOutputFormat::Text,
                threshold: None,
                inputs: vec!["left".to_string(), "right".to_string()],
            })
        );
    }

    #[test]
    fn parse_hash_many_and_xref_cover_profile_and_format_errors() {
        let error = parse(vec![
            "hash-many".to_string(),
            "--profile".to_string(),
            "wat".to_string(),
            "sample.bin".to_string(),
        ])
        .unwrap_err();
        assert_eq!(error, "unsupported profile: wat");

        let error = parse(vec!["hash-many".to_string(), "--profile".to_string()]).unwrap_err();
        assert_eq!(error, "missing value for --profile");

        let error = parse(vec![
            "hash-many".to_string(),
            "--format".to_string(),
            "wat".to_string(),
            "sample.bin".to_string(),
        ])
        .unwrap_err();
        assert_eq!(error, "unsupported format: wat");

        let error = parse(vec!["hash-many".to_string(), "--format".to_string()]).unwrap_err();
        assert_eq!(error, "missing value for --format");

        let error = parse(vec![
            "xref".to_string(),
            "--profile".to_string(),
            "wat".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
        .unwrap_err();
        assert_eq!(error, "unsupported profile: wat");
    }

    #[test]
    fn parse_help_routes_to_usage() {
        assert_eq!(parse(vec!["--help".to_string()]).unwrap_err(), usage());
        assert_eq!(parse(vec!["-h".to_string()]).unwrap_err(), usage());
        assert_eq!(parse(vec!["help".to_string()]).unwrap_err(), usage());
        assert!(is_root_help_flag("--help"));
        assert!(is_root_help_flag("-h"));
        assert!(is_root_help_flag("help"));
        assert!(!is_root_help_flag("hash"));
    }

    #[test]
    fn parse_rejects_unknown_options_and_missing_inputs() {
        assert!(
            parse(vec!["hash".to_string(), "--wat".to_string()])
                .unwrap_err()
                .contains("unknown option for hash")
        );
        assert!(
            parse(vec!["hash-many".to_string(), "--wat".to_string()])
                .unwrap_err()
                .contains("unknown option for hash-many")
        );
        assert!(
            parse(vec!["diff".to_string(), "--wat".to_string()])
                .unwrap_err()
                .contains("unknown option for diff")
        );
        assert!(
            parse(vec!["xref".to_string(), "--wat".to_string()])
                .unwrap_err()
                .contains("unknown option for xref")
        );
        assert_eq!(parse(vec!["hash".to_string()]).unwrap_err(), hash_usage());
        assert_eq!(
            parse(vec!["hash-many".to_string()]).unwrap_err(),
            hash_many_usage()
        );
        assert_eq!(
            parse(vec!["diff".to_string(), "left".to_string()]).unwrap_err(),
            diff_usage()
        );
        assert_eq!(
            parse(vec!["xref".to_string(), "left".to_string()]).unwrap_err(),
            xref_usage()
        );
    }

    #[test]
    fn parse_empty_args_and_specific_help_commands() {
        assert_eq!(parse(Vec::new()).unwrap_err(), usage());
        assert_eq!(
            parse(vec!["hash".to_string(), "--help".to_string()]).unwrap_err(),
            hash_usage()
        );
        assert_eq!(
            parse(vec!["hash-many".to_string(), "-h".to_string()]).unwrap_err(),
            hash_many_usage()
        );
        assert_eq!(
            parse(vec!["diff".to_string(), "--help".to_string()]).unwrap_err(),
            diff_usage()
        );
        assert_eq!(
            parse(vec!["xref".to_string(), "-h".to_string()]).unwrap_err(),
            xref_usage()
        );
        assert_eq!(
            parse_profile("wat").unwrap_err(),
            "unsupported profile: wat"
        );
        let mut cursor = ArgCursor::new(&[]);
        assert_eq!(cursor.next(), None);
    }

    #[test]
    fn option_helpers_cover_success_and_error_paths() {
        let args = vec!["128-1".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(
            parse_profile_option(&mut cursor).unwrap(),
            TlshProfile::standard_t1()
        );

        let args = vec!["sarif".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(
            parse_compare_format_option(&mut cursor).unwrap(),
            CompareOutputFormat::Sarif
        );

        let args = vec!["42".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(parse_threshold_option(&mut cursor).unwrap(), 42);

        let mut cursor = ArgCursor::new(&[]);
        assert_eq!(
            parse_profile_option(&mut cursor).unwrap_err(),
            "missing value for --profile"
        );
        let mut cursor = ArgCursor::new(&[]);
        assert_eq!(
            parse_compare_format_option(&mut cursor).unwrap_err(),
            "missing value for --format"
        );
        let mut cursor = ArgCursor::new(&[]);
        assert_eq!(
            parse_threshold_option(&mut cursor).unwrap_err(),
            "missing value for --threshold"
        );

        let args = vec!["wat".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(
            parse_profile_option(&mut cursor).unwrap_err(),
            "unsupported profile: wat"
        );
        let args = vec!["wat".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(
            parse_compare_format_option(&mut cursor).unwrap_err(),
            "unsupported format: wat"
        );
        let args = vec!["NaN".to_string()];
        let mut cursor = ArgCursor::new(&args);
        assert_eq!(
            parse_threshold_option(&mut cursor).unwrap_err(),
            "invalid threshold: NaN"
        );

        let args = vec!["value".to_string()];
        let mut index = 0usize;
        assert_eq!(
            require_slice_value(&args, &mut index, "--format").unwrap(),
            "value"
        );
        assert_eq!(index, 1);

        let args = Vec::<String>::new();
        let mut index = 0usize;
        assert_eq!(
            require_slice_value(&args, &mut index, "--format").unwrap_err(),
            "missing value for --format"
        );

        let args = vec!["128-1".to_string()];
        let mut index = 0usize;
        assert_eq!(
            take_hash_profile(&args, &mut index).unwrap(),
            TlshProfile::standard_t1()
        );

        let args = vec!["json".to_string()];
        let mut index = 0usize;
        assert_eq!(
            take_hash_format(&args, &mut index).unwrap(),
            HashOutputFormat::Json
        );

        let args = Vec::<String>::new();
        let mut index = 0usize;
        assert_eq!(
            take_hash_profile(&args, &mut index).unwrap_err(),
            "missing value for --profile"
        );

        let args = vec!["wat".to_string()];
        let mut index = 0usize;
        assert_eq!(
            take_hash_profile(&args, &mut index).unwrap_err(),
            "unsupported profile: wat"
        );

        let args = Vec::<String>::new();
        let mut index = 0usize;
        assert_eq!(
            take_hash_format(&args, &mut index).unwrap_err(),
            "missing value for --format"
        );

        let args = vec!["wat".to_string()];
        let mut index = 0usize;
        assert_eq!(
            take_hash_format(&args, &mut index).unwrap_err(),
            "unsupported format: wat"
        );
    }

    #[test]
    fn parse_rejects_invalid_formats_profiles_and_extra_arguments() {
        assert_eq!(
            parse(vec![
                "hash".to_string(),
                "--profile".to_string(),
                "wat".to_string(),
                "a".to_string(),
            ])
            .unwrap_err(),
            "unsupported profile: wat"
        );
        assert_eq!(
            parse(vec![
                "hash".to_string(),
                "--format".to_string(),
                "wat".to_string(),
                "a".to_string(),
            ])
            .unwrap_err(),
            "unsupported format: wat"
        );
        assert_eq!(
            parse(vec![
                "hash-many".to_string(),
                "--profile".to_string(),
                "wat".to_string(),
                "a".to_string(),
            ])
            .unwrap_err(),
            "unsupported profile: wat"
        );
        assert_eq!(
            parse(vec![
                "diff".to_string(),
                "--format".to_string(),
                "wat".to_string(),
                "a".to_string(),
                "b".to_string(),
            ])
            .unwrap_err(),
            "unsupported format: wat"
        );
        assert_eq!(
            parse(vec![
                "hash-many".to_string(),
                "--format".to_string(),
                "wat".to_string(),
                "a".to_string(),
            ])
            .unwrap_err(),
            "unsupported format: wat"
        );
        assert_eq!(
            parse(vec![
                "diff".to_string(),
                "--profile".to_string(),
                "wat".to_string(),
                "a".to_string(),
                "b".to_string(),
            ])
            .unwrap_err(),
            "unsupported profile: wat"
        );
        assert_eq!(
            parse(vec![
                "xref".to_string(),
                "--format".to_string(),
                "wat".to_string(),
                "a".to_string(),
                "b".to_string(),
            ])
            .unwrap_err(),
            "unsupported format: wat"
        );
        assert!(
            parse(vec!["hash".to_string(), "a".to_string(), "b".to_string()])
                .unwrap_err()
                .contains("unexpected extra argument: b")
        );
    }
}
