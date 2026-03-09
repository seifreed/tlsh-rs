use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BucketKind {
    Bucket128,
    Bucket256,
}

impl BucketKind {
    pub const fn effective_buckets(self) -> usize {
        match self {
            Self::Bucket128 => 128,
            Self::Bucket256 => 256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChecksumKind {
    OneByte,
    ThreeBytes,
}

impl ChecksumKind {
    pub const fn length(self) -> usize {
        match self {
            Self::OneByte => 1,
            Self::ThreeBytes => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TlshProfile {
    pub buckets: BucketKind,
    pub checksum: ChecksumKind,
}

impl TlshProfile {
    pub const fn new(buckets: BucketKind, checksum: ChecksumKind) -> Self {
        Self { buckets, checksum }
    }

    pub const fn standard_t1() -> Self {
        Self::new(BucketKind::Bucket128, ChecksumKind::OneByte)
    }

    pub const fn compact_128_3() -> Self {
        Self::new(BucketKind::Bucket128, ChecksumKind::ThreeBytes)
    }

    pub const fn full_256_1() -> Self {
        Self::new(BucketKind::Bucket256, ChecksumKind::OneByte)
    }

    pub const fn full_256_3() -> Self {
        Self::new(BucketKind::Bucket256, ChecksumKind::ThreeBytes)
    }

    pub const fn effective_buckets(self) -> usize {
        self.buckets.effective_buckets()
    }

    pub const fn code_size(self) -> usize {
        self.effective_buckets() / 4
    }

    pub const fn checksum_length(self) -> usize {
        self.checksum.length()
    }

    pub const fn raw_length(self) -> usize {
        self.code_size() * 2 + self.checksum_length() * 2 + 4
    }

    pub const fn encoded_length(self, with_version: bool) -> usize {
        self.raw_length() + if with_version { 2 } else { 0 }
    }

    pub const fn is_standard_t1(self) -> bool {
        match (self.buckets, self.checksum) {
            (BucketKind::Bucket128, ChecksumKind::OneByte) => true,
            (BucketKind::Bucket128, ChecksumKind::ThreeBytes) => false,
            (BucketKind::Bucket256, ChecksumKind::OneByte) => false,
            (BucketKind::Bucket256, ChecksumKind::ThreeBytes) => false,
        }
    }

    pub const fn from_raw_length(length: usize) -> Option<Self> {
        match length {
            70 => Some(Self::standard_t1()),
            74 => Some(Self::compact_128_3()),
            134 => Some(Self::full_256_1()),
            138 => Some(Self::full_256_3()),
            _ => None,
        }
    }

    pub fn from_cli_name(name: &str) -> Option<Self> {
        if name == "128-1" {
            return Some(Self::standard_t1());
        }
        if name == "128-3" {
            return Some(Self::compact_128_3());
        }
        if name == "256-1" {
            return Some(Self::full_256_1());
        }
        if name == "256-3" {
            return Some(Self::full_256_3());
        }
        None
    }

    pub const fn cli_name(self) -> &'static str {
        match self {
            Self {
                buckets: BucketKind::Bucket128,
                checksum: ChecksumKind::OneByte,
            } => "128-1",
            Self {
                buckets: BucketKind::Bucket128,
                checksum: ChecksumKind::ThreeBytes,
            } => "128-3",
            Self {
                buckets: BucketKind::Bucket256,
                checksum: ChecksumKind::OneByte,
            } => "256-1",
            Self {
                buckets: BucketKind::Bucket256,
                checksum: ChecksumKind::ThreeBytes,
            } => "256-3",
        }
    }
}

impl fmt::Display for TlshProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} buckets / {} byte checksum",
            self.effective_buckets(),
            self.checksum_length()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_and_checksum_sizes_match_expected_values() {
        assert_eq!(BucketKind::Bucket128.effective_buckets(), 128);
        assert_eq!(BucketKind::Bucket256.effective_buckets(), 256);
        assert_eq!(ChecksumKind::OneByte.length(), 1);
        assert_eq!(ChecksumKind::ThreeBytes.length(), 3);
    }

    #[test]
    fn profile_length_helpers_cover_all_profiles() {
        let standard = TlshProfile::standard_t1();
        assert_eq!(standard.code_size(), 32);
        assert_eq!(standard.checksum_length(), 1);
        assert_eq!(standard.raw_length(), 70);
        assert_eq!(standard.encoded_length(false), 70);
        assert_eq!(standard.encoded_length(true), 72);

        let compact = TlshProfile::compact_128_3();
        assert_eq!(compact.code_size(), 32);
        assert_eq!(compact.checksum_length(), 3);
        assert_eq!(compact.raw_length(), 74);
        assert_eq!(compact.encoded_length(false), 74);
        assert_eq!(compact.encoded_length(true), 76);

        let full_one = TlshProfile::full_256_1();
        assert_eq!(full_one.code_size(), 64);
        assert_eq!(full_one.checksum_length(), 1);
        assert_eq!(full_one.raw_length(), 134);
        assert_eq!(full_one.encoded_length(false), 134);
        assert_eq!(full_one.encoded_length(true), 136);

        let full_three = TlshProfile::full_256_3();
        assert_eq!(full_three.code_size(), 64);
        assert_eq!(full_three.checksum_length(), 3);
        assert_eq!(full_three.raw_length(), 138);
        assert_eq!(full_three.encoded_length(false), 138);
        assert_eq!(full_three.encoded_length(true), 140);
    }

    #[test]
    fn profile_roundtrips_cli_and_raw_lengths() {
        for (name, profile) in [
            ("128-1", TlshProfile::standard_t1()),
            ("128-3", TlshProfile::compact_128_3()),
            ("256-1", TlshProfile::full_256_1()),
            ("256-3", TlshProfile::full_256_3()),
        ] {
            assert_eq!(TlshProfile::from_cli_name(name), Some(profile));
            assert_eq!(profile.cli_name(), name);
            assert_eq!(
                TlshProfile::from_raw_length(profile.raw_length()),
                Some(profile)
            );
        }
        assert_eq!(TlshProfile::from_cli_name("wat"), None);
        assert_eq!(TlshProfile::from_raw_length(0), None);
    }

    #[test]
    fn standard_t1_detection_and_display_are_correct() {
        assert!(TlshProfile::standard_t1().is_standard_t1());
        assert!(!TlshProfile::compact_128_3().is_standard_t1());
        assert!(!TlshProfile::full_256_1().is_standard_t1());
        assert!(!TlshProfile::full_256_3().is_standard_t1());
        assert_eq!(
            TlshProfile::full_256_3().to_string(),
            "256 buckets / 3 byte checksum"
        );
    }
}
