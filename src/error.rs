use core::fmt;

use crate::profile::TlshProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlshError {
    DataTooLong,
    TooShort {
        min_length: usize,
        actual_length: usize,
    },
    InsufficientVariance,
    InvalidDigestLength {
        actual_length: usize,
    },
    InvalidDigestPrefix,
    InvalidHexCharacter {
        index: usize,
        byte: u8,
    },
    FileRead(String),
    StdinUnavailable,
    StdinAlreadyConsumed,
    IncompatibleProfiles {
        left: TlshProfile,
        right: TlshProfile,
    },
}

impl fmt::Display for TlshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataTooLong => write!(f, "data exceeds the maximum TLSH input length"),
            Self::TooShort {
                min_length,
                actual_length,
            } => write!(
                f,
                "input too short for TLSH: expected at least {min_length} bytes, got {actual_length}"
            ),
            Self::InsufficientVariance => {
                write!(
                    f,
                    "input does not contain enough variance to produce a TLSH digest"
                )
            }
            Self::InvalidDigestLength { actual_length } => {
                write!(f, "invalid TLSH digest length: {actual_length}")
            }
            Self::InvalidDigestPrefix => write!(f, "invalid TLSH digest prefix"),
            Self::InvalidHexCharacter { index, byte } => write!(
                f,
                "invalid hex character at index {index}: {:?}",
                *byte as char
            ),
            Self::FileRead(path) => write!(f, "unable to read file: {path}"),
            Self::StdinUnavailable => {
                write!(f, "stdin was requested but no stdin data was provided")
            }
            Self::StdinAlreadyConsumed => {
                write!(
                    f,
                    "stdin can only be consumed once in a single CLI invocation"
                )
            }
            Self::IncompatibleProfiles { left, right } => {
                write!(f, "incompatible TLSH profiles: {left} vs {right}")
            }
        }
    }
}

impl std::error::Error for TlshError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BucketKind, ChecksumKind};

    #[test]
    fn display_messages_cover_all_variants() {
        let cases = [
            (
                TlshError::DataTooLong,
                "data exceeds the maximum TLSH input length",
            ),
            (
                TlshError::TooShort {
                    min_length: 50,
                    actual_length: 10,
                },
                "input too short for TLSH: expected at least 50 bytes, got 10",
            ),
            (
                TlshError::InsufficientVariance,
                "input does not contain enough variance to produce a TLSH digest",
            ),
            (
                TlshError::InvalidDigestLength { actual_length: 7 },
                "invalid TLSH digest length: 7",
            ),
            (TlshError::InvalidDigestPrefix, "invalid TLSH digest prefix"),
            (
                TlshError::InvalidHexCharacter {
                    index: 2,
                    byte: b'Z',
                },
                "invalid hex character at index 2: 'Z'",
            ),
            (
                TlshError::FileRead("missing.bin".to_string()),
                "unable to read file: missing.bin",
            ),
            (
                TlshError::StdinUnavailable,
                "stdin was requested but no stdin data was provided",
            ),
            (
                TlshError::StdinAlreadyConsumed,
                "stdin can only be consumed once in a single CLI invocation",
            ),
            (
                TlshError::IncompatibleProfiles {
                    left: TlshProfile::standard_t1(),
                    right: TlshProfile::new(BucketKind::Bucket256, ChecksumKind::ThreeBytes),
                },
                "incompatible TLSH profiles: 128 buckets / 1 byte checksum vs 256 buckets / 3 byte checksum",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }
}
