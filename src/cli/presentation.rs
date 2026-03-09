use std::path::Path;

use super::model::{
    CompareOutputFormat, ComparisonReport, HashOutputFormat, HashRecord, Output, SimilarityFinding,
};

const SARIF_SCHEMA_URI: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const SARIF_RULE_ID: &str = "TLSH.Similarity";

pub fn render(output: Output) -> String {
    match output {
        Output::Hash(record, format) => render_hash(record, format),
        Output::HashMany(records, format) => render_hash_many(&records, format),
        Output::Diff(report, format) => render_report(report, format, true),
        Output::Xref(report, format) => render_report(report, format, false),
    }
}

fn render_hash(record: HashRecord, format: HashOutputFormat) -> String {
    match format {
        HashOutputFormat::Text => digest_value(&record),
        HashOutputFormat::Json => {
            let mut output = String::new();
            output.push('{');
            push_hash_record_json(&mut output, &record);
            output.push('}');
            output
        }
    }
}

fn render_hash_many(records: &[HashRecord], format: HashOutputFormat) -> String {
    match format {
        HashOutputFormat::Text => render_text_hash_many(records),
        HashOutputFormat::Json => render_json_hash_many(records),
    }
}

fn render_report(
    report: ComparisonReport,
    format: CompareOutputFormat,
    single_value: bool,
) -> String {
    match format {
        CompareOutputFormat::Text => {
            if single_value {
                report.findings[0].diff.to_string()
            } else {
                render_text_findings(&report.findings)
            }
        }
        CompareOutputFormat::Json => render_json_findings(&report),
        CompareOutputFormat::Sarif => render_sarif(&report),
    }
}

fn digest_value(record: &HashRecord) -> String {
    if record.raw {
        record.digest.raw_hex()
    } else {
        record.digest.encoded()
    }
}

fn render_text_hash_many(records: &[HashRecord]) -> String {
    let mut output = String::new();
    for (idx, record) in records.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        output.push_str(&digest_value(record));
        output.push('\t');
        output.push_str(&record.input);
    }
    output
}

fn render_text_findings(findings: &[SimilarityFinding]) -> String {
    let mut output = String::new();
    for (idx, finding) in findings.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        output.push_str(&finding.left_label);
        output.push('\t');
        output.push_str(&finding.right_label);
        output.push('\t');
        output.push_str(&finding.diff.to_string());
    }
    output
}

fn render_json_hash_many(records: &[HashRecord]) -> String {
    let mut output = String::new();
    output.push('[');
    for (idx, record) in records.iter().enumerate() {
        if idx > 0 {
            output.push(',');
        }
        output.push('{');
        push_hash_record_json(&mut output, record);
        output.push('}');
    }
    output.push(']');
    output
}

fn push_hash_record_json(output: &mut String, record: &HashRecord) {
    output.push_str("\"input\":");
    push_json_string(output, &record.input);
    output.push_str(",\"profile\":");
    push_json_string(output, record.profile.cli_name());
    output.push_str(",\"raw\":");
    output.push_str(if record.raw { "true" } else { "false" });
    output.push_str(",\"digest\":");
    push_json_string(output, &digest_value(record));
}

fn render_json_findings(report: &ComparisonReport) -> String {
    let mut output = String::new();
    output.push('{');
    output.push_str("\"profile\":");
    push_json_string(&mut output, report.profile.cli_name());
    output.push_str(",\"includeLength\":");
    output.push_str(if report.include_length {
        "true"
    } else {
        "false"
    });
    output.push_str(",\"results\":[");
    for (idx, finding) in report.findings.iter().enumerate() {
        if idx > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"left\":");
        push_json_string(&mut output, &finding.left_label);
        output.push_str(",\"right\":");
        push_json_string(&mut output, &finding.right_label);
        output.push_str(",\"diff\":");
        output.push_str(&finding.diff.to_string());
        output.push('}');
    }
    output.push_str("]}");
    output
}

fn render_sarif(report: &ComparisonReport) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str("  \"$schema\": ");
    push_json_string(&mut output, SARIF_SCHEMA_URI);
    output.push_str(",\n  \"version\": ");
    push_json_string(&mut output, SARIF_VERSION);
    output.push_str(",\n  \"runs\": [\n    {\n");
    output.push_str("      \"tool\": {\n        \"driver\": {\n");
    output.push_str("          \"name\": ");
    push_json_string(&mut output, "tlsh-rs");
    output.push_str(",\n          \"informationUri\": ");
    push_json_string(&mut output, "https://github.com/trendmicro/tlsh");
    output.push_str(",\n          \"rules\": [\n            {\n");
    output.push_str("              \"id\": ");
    push_json_string(&mut output, SARIF_RULE_ID);
    output.push_str(",\n              \"shortDescription\": { \"text\": ");
    push_json_string(&mut output, "TLSH similarity result");
    output.push_str(" },\n              \"fullDescription\": { \"text\": ");
    push_json_string(
        &mut output,
        "Reports the TLSH distance computed between two input artifacts or digests.",
    );
    output.push_str(" }\n            }\n          ]\n        }\n      },\n");
    output.push_str("      \"results\": [");
    if !report.findings.is_empty() {
        output.push('\n');
    }

    for (idx, finding) in report.findings.iter().enumerate() {
        if idx > 0 {
            output.push_str(",\n");
        }
        output.push_str("        {\n");
        output.push_str("          \"ruleId\": ");
        push_json_string(&mut output, SARIF_RULE_ID);
        output.push_str(",\n          \"level\": ");
        push_json_string(&mut output, "note");
        output.push_str(",\n          \"message\": { \"text\": ");
        push_json_string(
            &mut output,
            &format!(
                "TLSH distance between '{}' and '{}' is {}.",
                finding.left_label, finding.right_label, finding.diff
            ),
        );
        output.push_str(" },\n          \"locations\": [\n");
        output.push_str("            { \"physicalLocation\": { \"artifactLocation\": ");
        push_artifact_location(&mut output, &finding.left_label);
        output.push_str(" } },\n");
        output.push_str("            { \"physicalLocation\": { \"artifactLocation\": ");
        push_artifact_location(&mut output, &finding.right_label);
        output.push_str(" } }\n");
        output.push_str("          ],\n          \"properties\": {\n");
        output.push_str("            \"tlshDiff\": ");
        output.push_str(&finding.diff.to_string());
        output.push_str(",\n            \"profile\": ");
        push_json_string(&mut output, report.profile.cli_name());
        output.push_str(",\n            \"includeLength\": ");
        output.push_str(if report.include_length {
            "true"
        } else {
            "false"
        });
        output.push_str("\n          }\n        }");
    }

    if !report.findings.is_empty() {
        output.push('\n');
    }
    output.push_str("      ]\n    }\n  ]\n}");
    output
}

fn push_artifact_location(output: &mut String, label: &str) {
    if label == "-" || Path::new(label).exists() {
        output.push('{');
        output.push_str("\"uri\": ");
        push_json_string(output, label);
        output.push('}');
    } else {
        output.push('{');
        output.push_str("\"uri\": ");
        push_json_string(output, &format!("tlsh:{label}"));
        output.push_str(", \"description\": { \"text\": ");
        push_json_string(output, label);
        output.push_str(" }");
        output.push('}');
    }
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => {
                let escaped = format!("\\u{:04x}", c as u32);
                output.push_str(&escaped);
            }
            c => output.push(c),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TlshDigest;
    use crate::TlshProfile;

    fn digest() -> TlshDigest {
        TlshDigest::from_encoded(
            "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC",
        )
        .unwrap()
    }

    #[test]
    fn render_single_hash_json() {
        let output = render(Output::Hash(
            HashRecord {
                input: "sample.bin".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: false,
                digest: digest(),
            },
            HashOutputFormat::Json,
        ));
        assert!(output.contains("\"input\":\"sample.bin\""));
        assert!(output.contains("\"digest\":\"T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC\""));
    }

    #[test]
    fn render_empty_xref_text() {
        let output = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings: Vec::new(),
            },
            CompareOutputFormat::Text,
        ));
        assert_eq!(output, "");
    }

    #[test]
    fn render_sarif_for_digest_input_uses_tlsh_uri() {
        let output = render(Output::Diff(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings: vec![SimilarityFinding {
                    left_label: "digest".to_string(),
                    right_label: "other".to_string(),
                    diff: 7,
                }],
            },
            CompareOutputFormat::Sarif,
        ));
        assert!(output.contains("\"uri\": \"tlsh:digest\""));
        assert!(output.contains("\"includeLength\": true"));
    }

    #[test]
    fn render_sarif_multiple_findings_cover_separator_and_false_length() {
        let output = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                findings: vec![
                    SimilarityFinding {
                        left_label: "a".to_string(),
                        right_label: "b".to_string(),
                        diff: 7,
                    },
                    SimilarityFinding {
                        left_label: "c".to_string(),
                        right_label: "d".to_string(),
                        diff: 8,
                    },
                ],
            },
            CompareOutputFormat::Sarif,
        ));
        assert!(output.contains(",\n        {\n"));
        assert!(output.contains("\"includeLength\": false"));
    }

    #[test]
    fn render_json_escapes_control_characters() {
        let output = render(Output::Hash(
            HashRecord {
                input: "a\"\n\t".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: true,
                digest: digest(),
            },
            HashOutputFormat::Json,
        ));
        assert!(output.contains("a\\\"\\n\\t"));
    }

    #[test]
    fn render_text_variants_cover_hash_many_and_diff() {
        let digest = digest();
        let hash_many = render(Output::HashMany(
            vec![HashRecord {
                input: "a".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: false,
                digest: digest.clone(),
            }],
            HashOutputFormat::Text,
        ));
        assert_eq!(hash_many, format!("{}\ta", digest.encoded()));

        let diff = render(Output::Diff(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings: vec![SimilarityFinding {
                    left_label: "a".to_string(),
                    right_label: "b".to_string(),
                    diff: 9,
                }],
            },
            CompareOutputFormat::Text,
        ));
        assert_eq!(diff, "9");
    }

    #[test]
    fn render_json_findings_and_file_sarif_locations() {
        let file_path = format!("{}/fixtures/small.txt", env!("CARGO_MANIFEST_DIR"));
        let json = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                findings: vec![SimilarityFinding {
                    left_label: "a".to_string(),
                    right_label: "b".to_string(),
                    diff: 4,
                }],
            },
            CompareOutputFormat::Json,
        ));
        assert!(json.contains("\"includeLength\":false"));
        assert!(json.contains("\"diff\":4"));

        let sarif = render(Output::Diff(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings: vec![SimilarityFinding {
                    left_label: file_path.clone(),
                    right_label: file_path,
                    diff: 0,
                }],
            },
            CompareOutputFormat::Sarif,
        ));
        assert!(sarif.contains("\"uri\": "));
        assert!(!sarif.contains("tlsh:"));
    }

    #[test]
    fn render_hash_text_and_hash_many_json() {
        let digest = digest();
        let text = render(Output::Hash(
            HashRecord {
                input: "raw.bin".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: true,
                digest: digest.clone(),
            },
            HashOutputFormat::Text,
        ));
        assert_eq!(text, digest.raw_hex());

        let json = render(Output::HashMany(
            vec![HashRecord {
                input: "raw.bin".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: true,
                digest,
            }],
            HashOutputFormat::Json,
        ));
        assert!(json.starts_with('['));
        assert!(json.contains("\"raw\":true"));
    }

    #[test]
    fn render_multiple_findings_cover_separators_and_empty_sarif_results() {
        let findings = vec![
            SimilarityFinding {
                left_label: "a".to_string(),
                right_label: "b".to_string(),
                diff: 1,
            },
            SimilarityFinding {
                left_label: "c".to_string(),
                right_label: "d".to_string(),
                diff: 2,
            },
        ];

        let text = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings: findings.clone(),
            },
            CompareOutputFormat::Text,
        ));
        assert!(text.contains('\n'));

        let json = render(Output::HashMany(
            vec![
                HashRecord {
                    input: "a".to_string(),
                    profile: TlshProfile::standard_t1(),
                    raw: false,
                    digest: digest(),
                },
                HashRecord {
                    input: "b".to_string(),
                    profile: TlshProfile::standard_t1(),
                    raw: false,
                    digest: digest(),
                },
            ],
            HashOutputFormat::Json,
        ));
        assert!(json.contains("},{"));

        let json_findings = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: true,
                findings,
            },
            CompareOutputFormat::Json,
        ));
        assert!(json_findings.contains("\"includeLength\":true"));
        assert!(json_findings.contains("},{"));

        let sarif = render(Output::Xref(
            ComparisonReport {
                profile: TlshProfile::standard_t1(),
                include_length: false,
                findings: Vec::new(),
            },
            CompareOutputFormat::Sarif,
        ));
        assert!(sarif.contains("\"results\": ["));
        assert!(
            sarif.contains("\"includeLength\": false") || !sarif.contains("\"includeLength\":")
        );

        let escaped = render(Output::Hash(
            HashRecord {
                input: "a\\\r\u{0001}".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: false,
                digest: digest(),
            },
            HashOutputFormat::Json,
        ));
        assert!(escaped.contains("\\\\"));
        assert!(escaped.contains("\\r"));
        assert!(escaped.contains("\\u0001"));
    }

    #[test]
    fn direct_render_helpers_cover_remaining_branches() {
        let records = vec![
            HashRecord {
                input: "a".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: false,
                digest: digest(),
            },
            HashRecord {
                input: "b".to_string(),
                profile: TlshProfile::standard_t1(),
                raw: true,
                digest: digest(),
            },
        ];
        let text_many = render_text_hash_many(&records);
        assert!(text_many.contains('\n'));

        let findings = vec![
            SimilarityFinding {
                left_label: "a".to_string(),
                right_label: "b".to_string(),
                diff: 1,
            },
            SimilarityFinding {
                left_label: "c".to_string(),
                right_label: "d".to_string(),
                diff: 2,
            },
        ];
        let text_findings = render_text_findings(&findings);
        assert!(text_findings.contains('\n'));

        let json = render_json_findings(&ComparisonReport {
            profile: TlshProfile::standard_t1(),
            include_length: true,
            findings: findings.clone(),
        });
        assert!(json.contains("\"includeLength\":true"));
        assert!(json.contains("},{"));

        let empty_sarif = render_sarif(&ComparisonReport {
            profile: TlshProfile::standard_t1(),
            include_length: false,
            findings: Vec::new(),
        });
        assert!(empty_sarif.contains("\"results\": ["));
        assert!(!empty_sarif.contains("\"includeLength\": false"));

        let mut out = String::new();
        push_json_string(&mut out, "\\\r\u{0001}");
        assert_eq!(out, "\"\\\\\\r\\u0001\"");
    }
}
