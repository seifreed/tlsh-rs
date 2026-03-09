use crate::{TlshDigest, TlshProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashOutputFormat {
    Text,
    Json,
}

impl HashOutputFormat {
    pub fn from_cli_name(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOutputFormat {
    Text,
    Json,
    Sarif,
}

impl CompareOutputFormat {
    pub fn from_cli_name(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Hash(HashCommand),
    HashMany(HashManyCommand),
    Diff(DiffCommand),
    Xref(XrefCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashCommand {
    pub profile: TlshProfile,
    pub raw: bool,
    pub format: HashOutputFormat,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashManyCommand {
    pub profile: TlshProfile,
    pub raw: bool,
    pub format: HashOutputFormat,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffCommand {
    pub profile: TlshProfile,
    pub include_length: bool,
    pub format: CompareOutputFormat,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrefCommand {
    pub profile: TlshProfile,
    pub include_length: bool,
    pub format: CompareOutputFormat,
    pub threshold: Option<i32>,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashRecord {
    pub input: String,
    pub profile: TlshProfile,
    pub raw: bool,
    pub digest: TlshDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityFinding {
    pub left_label: String,
    pub right_label: String,
    pub diff: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonReport {
    pub profile: TlshProfile,
    pub include_length: bool,
    pub findings: Vec<SimilarityFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Hash(HashRecord, HashOutputFormat),
    HashMany(Vec<HashRecord>, HashOutputFormat),
    Diff(ComparisonReport, CompareOutputFormat),
    Xref(ComparisonReport, CompareOutputFormat),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_formats_parse_cli_names() {
        assert_eq!(
            HashOutputFormat::from_cli_name("text"),
            Some(HashOutputFormat::Text)
        );
        assert_eq!(
            HashOutputFormat::from_cli_name("json"),
            Some(HashOutputFormat::Json)
        );
        assert_eq!(HashOutputFormat::from_cli_name("sarif"), None);
        assert_eq!(
            CompareOutputFormat::from_cli_name("text"),
            Some(CompareOutputFormat::Text)
        );
        assert_eq!(
            CompareOutputFormat::from_cli_name("json"),
            Some(CompareOutputFormat::Json)
        );
        assert_eq!(
            CompareOutputFormat::from_cli_name("sarif"),
            Some(CompareOutputFormat::Sarif)
        );
        assert_eq!(CompareOutputFormat::from_cli_name("wat"), None);
    }
}
