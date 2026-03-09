use tlsh_rs::cli;

const SMALL_HASH: &str = "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";
const SMALL2_HASH: &str =
    "T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";
const FULL_256_3_SMALL: &str = "F8F43EA0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD";

#[test]
fn cli_hash_outputs_standard_digest() {
    let output = run(&["hash", fixture("small.txt").as_str()]);
    assert_eq!(output.trim(), SMALL_HASH);
}

#[test]
fn cli_hash_supports_raw_and_profile() {
    let output = run(&[
        "hash",
        "--profile",
        "256-3",
        "--raw",
        fixture("small.txt").as_str(),
    ]);
    assert_eq!(output.trim(), FULL_256_3_SMALL);
}

#[test]
fn cli_hash_supports_json_output() {
    let output = run(&["hash", "--format", "json", fixture("small.txt").as_str()]);
    assert!(output.contains("\"input\":"));
    assert!(output.contains("\"profile\":\"128-1\""));
    assert!(output.contains(SMALL_HASH));
}

#[test]
fn cli_hash_supports_stdin_marker() {
    let data = std::fs::read(fixture("small.txt")).unwrap();
    let output = run_with_stdin(&["hash", "-"], &data);
    assert_eq!(output.trim(), SMALL_HASH);
}

#[test]
fn cli_diff_supports_files() {
    let output = run(&[
        "diff",
        fixture("small.txt").as_str(),
        fixture("small2.txt").as_str(),
    ]);
    assert_eq!(output.trim(), "221");
}

#[test]
fn cli_diff_supports_digest_inputs() {
    let output = run(&["diff", SMALL_HASH, SMALL2_HASH]);
    assert_eq!(output.trim(), "221");
}

#[test]
fn cli_diff_supports_json_output() {
    let output = run(&["diff", "--format", "json", SMALL_HASH, SMALL2_HASH]);
    assert!(output.contains("\"profile\":\"128-1\""));
    assert!(output.contains("\"diff\":221"));
}

#[test]
fn cli_hash_many_outputs_tsv_lines() {
    let small = fixture("small.txt");
    let small2 = fixture("small2.txt");
    let output = run(&["hash-many", small.as_str(), small2.as_str()]);
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], format!("{SMALL_HASH}\t{small}"));
    assert_eq!(lines[1], format!("{SMALL2_HASH}\t{small2}"));
}

#[test]
fn cli_hash_many_supports_json_output() {
    let small = fixture("small.txt");
    let small2 = fixture("small2.txt");
    let output = run(&[
        "hash-many",
        "--format",
        "json",
        small.as_str(),
        small2.as_str(),
    ]);
    assert!(output.starts_with('['));
    assert!(output.contains(SMALL_HASH));
    assert!(output.contains(SMALL2_HASH));
}

#[test]
fn cli_xref_outputs_pairwise_distances() {
    let small = fixture("small.txt");
    let small2 = fixture("small2.txt");
    let output = run(&["xref", small.as_str(), small2.as_str(), SMALL_HASH]);
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], format!("{small}\t{small2}\t221"));
    assert_eq!(lines[1], format!("{small}\t{SMALL_HASH}\t0"));
    assert_eq!(lines[2], format!("{small2}\t{SMALL_HASH}\t221"));
}

#[test]
fn cli_xref_supports_threshold_filtering() {
    let small = fixture("small.txt");
    let small2 = fixture("small2.txt");
    let output = run(&[
        "xref",
        "--threshold",
        "10",
        small.as_str(),
        small2.as_str(),
        SMALL_HASH,
    ]);
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], format!("{small}\t{SMALL_HASH}\t0"));
}

#[test]
fn cli_xref_supports_json_output() {
    let small = fixture("small.txt");
    let small2 = fixture("small2.txt");
    let output = run(&[
        "xref",
        "--format",
        "json",
        "--threshold",
        "10",
        small.as_str(),
        small2.as_str(),
        SMALL_HASH,
    ]);
    assert!(output.contains("\"results\":["));
    assert!(output.contains("\"diff\":0"));
}

#[test]
fn cli_diff_supports_sarif_output() {
    let output = run(&[
        "diff",
        "--format",
        "sarif",
        fixture("small.txt").as_str(),
        fixture("small2.txt").as_str(),
    ]);
    assert!(output.contains("\"version\": \"2.1.0\""));
    assert!(output.contains("\"$schema\":"));
    assert!(output.contains("\"ruleId\": \"TLSH.Similarity\""));
    assert!(output.contains("\"tlshDiff\": 221"));
    assert!(output.contains("\"profile\": \"128-1\""));
}

#[test]
fn cli_xref_supports_sarif_output() {
    let output = run(&[
        "xref",
        "--format",
        "sarif",
        "--threshold",
        "10",
        fixture("small.txt").as_str(),
        fixture("small2.txt").as_str(),
        SMALL_HASH,
    ]);
    assert!(output.contains("\"version\": \"2.1.0\""));
    assert!(output.contains("\"results\": ["));
    assert!(output.contains("\"tlshDiff\": 0"));
    assert!(output.contains("\"includeLength\": true"));
}

fn run(args: &[&str]) -> String {
    cli::run(args.iter().map(|value| (*value).to_string()).collect()).unwrap()
}

fn run_with_stdin(args: &[&str], stdin: &[u8]) -> String {
    cli::run_with_stdin(
        args.iter().map(|value| (*value).to_string()).collect(),
        Some(stdin),
    )
    .unwrap()
}

fn fixture(name: &str) -> String {
    format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}
