use crate::TlshError;

use super::io::CliContext;
use super::model::{Command, ComparisonReport, HashRecord, Output, SimilarityFinding};

pub fn execute(command: Command, context: &mut CliContext<'_>) -> Result<Output, TlshError> {
    match command {
        Command::Hash(command) => execute_hash(command, context),
        Command::HashMany(command) => execute_hash_many(command, context),
        Command::Diff(command) => execute_diff(command, context),
        Command::Xref(command) => execute_xref(command, context),
    }
}

#[allow(clippy::question_mark)]
fn execute_hash(
    command: super::model::HashCommand,
    context: &mut CliContext<'_>,
) -> Result<Output, TlshError> {
    let digest = match context.hash_input(&command.input, command.profile) {
        Ok(digest) => digest,
        Err(error) => return Err(error),
    };

    Ok(Output::Hash(
        HashRecord {
            input: command.input,
            profile: command.profile,
            raw: command.raw,
            digest,
        },
        command.format,
    ))
}

#[allow(clippy::question_mark)]
fn execute_hash_many(
    command: super::model::HashManyCommand,
    context: &mut CliContext<'_>,
) -> Result<Output, TlshError> {
    let mut records = Vec::with_capacity(command.inputs.len());
    for input in command.inputs {
        let digest = match context.hash_input(&input, command.profile) {
            Ok(digest) => digest,
            Err(error) => return Err(error),
        };
        records.push(HashRecord {
            input,
            profile: command.profile,
            raw: command.raw,
            digest,
        });
    }

    Ok(Output::HashMany(records, command.format))
}

#[allow(clippy::question_mark)]
fn execute_diff(
    command: super::model::DiffCommand,
    context: &mut CliContext<'_>,
) -> Result<Output, TlshError> {
    let left = match context.load_input(&command.left, command.profile) {
        Ok(digest) => digest,
        Err(error) => return Err(error),
    };
    let right = match context.load_input(&command.right, command.profile) {
        Ok(digest) => digest,
        Err(error) => return Err(error),
    };
    let diff = match compare_pair(&left, &right, command.include_length) {
        Ok(diff) => diff,
        Err(error) => return Err(error),
    };

    Ok(Output::Diff(
        ComparisonReport {
            profile: command.profile,
            include_length: command.include_length,
            findings: vec![SimilarityFinding {
                left_label: command.left,
                right_label: command.right,
                diff,
            }],
        },
        command.format,
    ))
}

#[allow(clippy::question_mark)]
fn execute_xref(
    command: super::model::XrefCommand,
    context: &mut CliContext<'_>,
) -> Result<Output, TlshError> {
    let mut entries = Vec::with_capacity(command.inputs.len());
    for input in &command.inputs {
        let digest = match context.load_input(input, command.profile) {
            Ok(digest) => digest,
            Err(error) => return Err(error),
        };
        entries.push((input.clone(), digest));
    }

    let mut findings = Vec::new();
    for left_idx in 0..entries.len() {
        for right_idx in (left_idx + 1)..entries.len() {
            let diff = match compare_pair(
                &entries[left_idx].1,
                &entries[right_idx].1,
                command.include_length,
            ) {
                Ok(diff) => diff,
                Err(error) => return Err(error),
            };

            if exceeds_threshold(diff, command.threshold) {
                continue;
            }

            findings.push(SimilarityFinding {
                left_label: entries[left_idx].0.clone(),
                right_label: entries[right_idx].0.clone(),
                diff,
            });
        }
    }

    Ok(Output::Xref(
        ComparisonReport {
            profile: command.profile,
            include_length: command.include_length,
            findings,
        },
        command.format,
    ))
}

fn compare_pair(
    left: &crate::TlshDigest,
    right: &crate::TlshDigest,
    include_length: bool,
) -> Result<i32, TlshError> {
    if include_length {
        return left.try_diff(right);
    }
    left.try_diff_no_length(right)
}

fn exceeds_threshold(diff: i32, threshold: Option<i32>) -> bool {
    match threshold {
        Some(limit) => diff > limit,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TlshDigest;
    use crate::TlshProfile;
    use crate::cli::model::{
        Command, CompareOutputFormat, DiffCommand, HashCommand, HashManyCommand, HashOutputFormat,
        XrefCommand,
    };

    const STANDARD_SMALL: &str =
        "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";
    const STANDARD_SMALL2: &str =
        "T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

    fn fixture(name: &str) -> String {
        format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    fn manifest_dir() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    fn digest(encoded: &str) -> TlshDigest {
        TlshDigest::from_encoded(encoded).unwrap()
    }

    #[test]
    fn execute_hash_command_returns_hash_output() {
        let input = fixture("small.txt");
        let mut context = CliContext::new(None);
        let output = execute(
            Command::Hash(HashCommand {
                profile: TlshProfile::standard_t1(),
                raw: false,
                format: HashOutputFormat::Text,
                input: input.clone(),
            }),
            &mut context,
        )
        .unwrap();

        assert_eq!(
            output,
            Output::Hash(
                HashRecord {
                    input,
                    profile: TlshProfile::standard_t1(),
                    raw: false,
                    digest: digest(STANDARD_SMALL),
                },
                HashOutputFormat::Text,
            )
        );
    }

    #[test]
    fn execute_hash_many_command_returns_multiple_records() {
        let small = fixture("small.txt");
        let small2 = fixture("small2.txt");
        let mut context = CliContext::new(None);
        let output = execute(
            Command::HashMany(HashManyCommand {
                profile: TlshProfile::standard_t1(),
                raw: true,
                format: HashOutputFormat::Json,
                inputs: vec![small.clone(), small2.clone()],
            }),
            &mut context,
        )
        .unwrap();

        assert_eq!(
            output,
            Output::HashMany(
                vec![
                    HashRecord {
                        input: small,
                        profile: TlshProfile::standard_t1(),
                        raw: true,
                        digest: digest(STANDARD_SMALL),
                    },
                    HashRecord {
                        input: small2,
                        profile: TlshProfile::standard_t1(),
                        raw: true,
                        digest: digest(STANDARD_SMALL2),
                    },
                ],
                HashOutputFormat::Json,
            )
        );
    }

    #[test]
    fn execute_diff_command_supports_no_length() {
        let left = fixture("small.txt");
        let right = fixture("small2.txt");
        let mut context = CliContext::new(None);
        let output = execute(
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                format: CompareOutputFormat::Json,
                left: left.clone(),
                right: right.clone(),
            }),
            &mut context,
        )
        .unwrap();

        assert_eq!(
            output,
            Output::Diff(
                ComparisonReport {
                    profile: TlshProfile::standard_t1(),
                    include_length: false,
                    findings: vec![SimilarityFinding {
                        left_label: left,
                        right_label: right,
                        diff: 221,
                    }],
                },
                CompareOutputFormat::Json,
            )
        );
    }

    #[test]
    fn execute_xref_command_applies_threshold() {
        let small = fixture("small.txt");
        let small2 = fixture("small2.txt");
        let mut context = CliContext::new(None);
        let output = execute(
            Command::Xref(XrefCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                threshold: Some(0),
                inputs: vec![small.clone(), small2, STANDARD_SMALL.to_string()],
            }),
            &mut context,
        )
        .unwrap();

        assert_eq!(
            output,
            Output::Xref(
                ComparisonReport {
                    profile: TlshProfile::standard_t1(),
                    include_length: true,
                    findings: vec![SimilarityFinding {
                        left_label: small,
                        right_label: STANDARD_SMALL.to_string(),
                        diff: 0,
                    }],
                },
                CompareOutputFormat::Text,
            )
        );
    }

    #[test]
    fn execute_propagates_input_errors() {
        let mut context = CliContext::new(None);
        let error = execute(
            Command::Hash(HashCommand {
                profile: TlshProfile::standard_t1(),
                raw: false,
                format: HashOutputFormat::Text,
                input: "-".to_string(),
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(error, TlshError::StdinUnavailable);
    }

    #[test]
    fn execute_hash_many_propagates_hash_errors() {
        let mut context = CliContext::new(None);
        let error = execute(
            Command::HashMany(HashManyCommand {
                profile: TlshProfile::standard_t1(),
                raw: false,
                format: HashOutputFormat::Text,
                inputs: vec!["-".to_string()],
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(error, TlshError::StdinUnavailable);
    }

    #[test]
    fn execute_diff_propagates_load_and_compare_errors() {
        let mut context = CliContext::new(None);
        let missing_left = execute(
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                left: manifest_dir(),
                right: fixture("small.txt"),
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(missing_left, TlshError::FileRead(manifest_dir()));

        let mut context = CliContext::new(None);
        let missing_right = execute(
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                left: fixture("small.txt"),
                right: manifest_dir(),
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(missing_right, TlshError::FileRead(manifest_dir()));

        let mut context = CliContext::new(None);
        let incompatible = execute(
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                left: fixture("small.txt"),
                right: "F8F43EA0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD".to_string(),
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(
            incompatible,
            TlshError::IncompatibleProfiles {
                left: TlshProfile::standard_t1(),
                right: TlshProfile::full_256_3(),
            }
        );
    }

    #[test]
    fn execute_xref_propagates_load_and_compare_errors() {
        let mut context = CliContext::new(None);
        let missing_input = execute(
            Command::Xref(XrefCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                threshold: None,
                inputs: vec![fixture("small.txt"), manifest_dir()],
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(missing_input, TlshError::FileRead(manifest_dir()));

        let mut context = CliContext::new(None);
        let incompatible = execute(
            Command::Xref(XrefCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                threshold: None,
                inputs: vec![
                    fixture("small.txt"),
                    "F8F43EA0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD".to_string(),
                ],
            }),
            &mut context,
        )
        .unwrap_err();
        assert_eq!(
            incompatible,
            TlshError::IncompatibleProfiles {
                left: TlshProfile::standard_t1(),
                right: TlshProfile::full_256_3(),
            }
        );
    }

    #[test]
    fn execute_diff_command_with_length_and_xref_without_length() {
        let left = fixture("small.txt");
        let right = fixture("small2.txt");
        let mut context = CliContext::new(None);
        let diff = execute(
            Command::Diff(DiffCommand {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                format: CompareOutputFormat::Text,
                left: left.clone(),
                right: right.clone(),
            }),
            &mut context,
        )
        .unwrap();
        assert_eq!(
            diff,
            Output::Diff(
                ComparisonReport {
                    profile: TlshProfile::standard_t1(),
                    include_length: true,
                    findings: vec![SimilarityFinding {
                        left_label: left.clone(),
                        right_label: right.clone(),
                        diff: 221,
                    }],
                },
                CompareOutputFormat::Text,
            )
        );

        let mut context = CliContext::new(None);
        let xref = execute(
            Command::Xref(XrefCommand {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                format: CompareOutputFormat::Json,
                threshold: None,
                inputs: vec![left.clone(), right.clone()],
            }),
            &mut context,
        )
        .unwrap();
        assert_eq!(
            xref,
            Output::Xref(
                ComparisonReport {
                    profile: TlshProfile::standard_t1(),
                    include_length: false,
                    findings: vec![SimilarityFinding {
                        left_label: left,
                        right_label: right,
                        diff: 221,
                    }],
                },
                CompareOutputFormat::Json,
            )
        );
    }

    #[test]
    fn compare_pair_and_threshold_helpers_cover_both_paths() {
        let left = digest(STANDARD_SMALL);
        let right = digest(STANDARD_SMALL2);

        assert_eq!(compare_pair(&left, &right, true).unwrap(), 221);
        assert_eq!(compare_pair(&left, &right, false).unwrap(), 221);
        assert!(exceeds_threshold(10, Some(5)));
        assert!(!exceeds_threshold(10, Some(10)));
        assert!(!exceeds_threshold(10, None));
    }
}
