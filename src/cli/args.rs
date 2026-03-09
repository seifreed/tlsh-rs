use crate::TlshProfile;

use super::model::{
    Command, CompareOutputFormat, DiffCommand, HashCommand, HashManyCommand, HashOutputFormat,
    XrefCommand,
};

pub fn parse(args: Vec<String>) -> Result<Command, String> {
    let (command, rest) = match args.split_first() {
        Some(pair) => pair,
        None => return Err(usage()),
    };

    match command.as_str() {
        "hash" => parse_hash(rest),
        "hash-many" => parse_hash_many(rest),
        "diff" => parse_diff(rest),
        "xref" => parse_xref(rest),
        "--help" | "-h" | "help" => Err(usage()),
        other => Err(format!("unknown command: {other}\n\n{}", usage())),
    }
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

fn parse_hash(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut raw = false;
    let mut format = HashOutputFormat::Text;
    let mut input = None::<String>;
    let mut parser = ArgCursor::new(args);

    while let Some(arg) = parser.next() {
        match arg {
            "--profile" => {
                let value = parser.require_value("--profile")?;
                profile = parse_profile(value)?;
            }
            "--raw" => raw = true,
            "--format" => {
                let value = parser.require_value("--format")?;
                format = match HashOutputFormat::from_cli_name(value) {
                    Some(format) => format,
                    None => return Err(format!("unsupported format: {value}")),
                };
            }
            "--help" | "-h" => return Err(hash_usage()),
            value if value.starts_with("--") => {
                return Err(format!(
                    "unknown option for hash: {value}\n\n{}",
                    hash_usage()
                ));
            }
            value => {
                if input.is_some() {
                    return Err(format!(
                        "unexpected extra argument: {value}\n\n{}",
                        hash_usage()
                    ));
                }
                input = Some(value.to_string());
            }
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

fn parse_hash_many(args: &[String]) -> Result<Command, String> {
    let mut profile = TlshProfile::standard_t1();
    let mut raw = false;
    let mut format = HashOutputFormat::Text;
    let mut inputs = Vec::new();
    let mut parser = ArgCursor::new(args);

    while let Some(arg) = parser.next() {
        match arg {
            "--profile" => {
                let value = parser.require_value("--profile")?;
                profile = parse_profile(value)?;
            }
            "--raw" => raw = true,
            "--format" => {
                let value = parser.require_value("--format")?;
                format = match HashOutputFormat::from_cli_name(value) {
                    Some(format) => format,
                    None => return Err(format!("unsupported format: {value}")),
                };
            }
            "--help" | "-h" => return Err(hash_many_usage()),
            value if value.starts_with("--") => {
                return Err(format!(
                    "unknown option for hash-many: {value}\n\n{}",
                    hash_many_usage()
                ));
            }
            value => inputs.push(value.to_string()),
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
        match arg {
            "--profile" => {
                let value = parser.require_value("--profile")?;
                profile = parse_profile(value)?;
            }
            "--no-length" => include_length = false,
            "--format" => {
                let value = parser.require_value("--format")?;
                format = match CompareOutputFormat::from_cli_name(value) {
                    Some(format) => format,
                    None => return Err(format!("unsupported format: {value}")),
                };
            }
            "--help" | "-h" => return Err(diff_usage()),
            value if value.starts_with("--") => {
                return Err(format!(
                    "unknown option for diff: {value}\n\n{}",
                    diff_usage()
                ));
            }
            value => values.push(value.to_string()),
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
        match arg {
            "--profile" => {
                let value = parser.require_value("--profile")?;
                profile = parse_profile(value)?;
            }
            "--no-length" => include_length = false,
            "--format" => {
                let value = parser.require_value("--format")?;
                format = match CompareOutputFormat::from_cli_name(value) {
                    Some(format) => format,
                    None => return Err(format!("unsupported format: {value}")),
                };
            }
            "--threshold" => {
                let value = parser.require_value("--threshold")?;
                threshold = Some(
                    value
                        .parse::<i32>()
                        .map_err(|_| format!("invalid threshold: {value}"))?,
                );
            }
            "--help" | "-h" => return Err(xref_usage()),
            value if value.starts_with("--") => {
                return Err(format!(
                    "unknown option for xref: {value}\n\n{}",
                    xref_usage()
                ));
            }
            value => inputs.push(value.to_string()),
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
    fn parse_help_routes_to_usage() {
        assert_eq!(parse(vec!["help".to_string()]).unwrap_err(), usage());
    }

    #[test]
    fn parse_hash_many_and_xref_commands() {
        let hash_many = parse(vec![
            "hash-many".to_string(),
            "--raw".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "a".to_string(),
            "b".to_string(),
        ])
        .unwrap();
        assert_eq!(
            hash_many,
            Command::HashMany(HashManyCommand {
                profile: TlshProfile::standard_t1(),
                raw: true,
                format: HashOutputFormat::Json,
                inputs: vec!["a".to_string(), "b".to_string()],
            })
        );

        let xref = parse(vec![
            "xref".to_string(),
            "--profile".to_string(),
            "256-1".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--threshold".to_string(),
            "8".to_string(),
            "a".to_string(),
            "b".to_string(),
        ])
        .unwrap();
        assert_eq!(
            xref,
            Command::Xref(XrefCommand {
                profile: TlshProfile::full_256_1(),
                include_length: true,
                format: CompareOutputFormat::Json,
                threshold: Some(8),
                inputs: vec!["a".to_string(), "b".to_string()],
            })
        );
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
    fn parse_rejects_invalid_formats_profiles_and_extra_arguments() {
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
            parse(vec!["hash".to_string(), "a".to_string(), "b".to_string(),])
                .unwrap_err()
                .contains("unexpected extra argument: b")
        );
    }
}
