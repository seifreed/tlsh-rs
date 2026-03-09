use core::fmt;
use core::str::FromStr;

use crate::error::TlshError;
use crate::internal::constants::{
    HEX_CHARS, LENGTH_MULTIPLIER, RANGE_LVALUE, RANGE_QRATIO, T1_PREFIX,
};
use crate::profile::TlshProfile;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TlshDigest {
    profile: TlshProfile,
    checksum: Vec<u8>,
    lvalue: u8,
    q1_ratio: u8,
    q2_ratio: u8,
    code: Vec<u8>,
}

impl TlshDigest {
    pub(crate) fn new(
        profile: TlshProfile,
        checksum: Vec<u8>,
        lvalue: u8,
        q1_ratio: u8,
        q2_ratio: u8,
        code: Vec<u8>,
    ) -> Self {
        Self {
            profile,
            checksum,
            lvalue,
            q1_ratio,
            q2_ratio,
            code,
        }
    }

    pub fn profile(&self) -> TlshProfile {
        self.profile
    }

    pub fn checksum(&self) -> &[u8] {
        &self.checksum
    }

    pub fn lvalue(&self) -> u8 {
        self.lvalue
    }

    pub fn q1_ratio(&self) -> u8 {
        self.q1_ratio
    }

    pub fn q2_ratio(&self) -> u8 {
        self.q2_ratio
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn bucket_value(&self, bucket: usize) -> Option<u8> {
        if bucket >= self.profile.effective_buckets() {
            return None;
        }

        let idx = (self.profile.code_size() - (bucket / 4)) - 1;
        let elem = bucket % 4;
        let value = self.code[idx];
        Some((value >> (elem * 2)) & 0b11)
    }

    pub fn encoded(&self) -> String {
        self.encoded_with_version(self.profile.is_standard_t1())
    }

    pub fn encoded_with_version(&self, with_version: bool) -> String {
        let mut out = String::with_capacity(self.profile.encoded_length(with_version));
        if with_version {
            out.push_str(T1_PREFIX);
            self.write_raw_hex(&mut out);
            return out;
        }
        self.write_raw_hex(&mut out);
        out
    }

    pub fn raw_hex(&self) -> String {
        let mut out = String::with_capacity(self.profile.raw_length());
        self.write_raw_hex(&mut out);
        out
    }

    pub fn diff(&self, other: &Self) -> i32 {
        self.try_diff(other).expect("incompatible TLSH profiles")
    }

    pub fn diff_no_length(&self, other: &Self) -> i32 {
        self.try_diff_no_length(other)
            .expect("incompatible TLSH profiles")
    }

    pub fn try_diff(&self, other: &Self) -> Result<i32, TlshError> {
        self.try_diff_with_options(other, true)
    }

    pub fn try_diff_no_length(&self, other: &Self) -> Result<i32, TlshError> {
        self.try_diff_with_options(other, false)
    }

    pub fn from_encoded(input: &str) -> Result<Self, TlshError> {
        if let Some(raw) = input.strip_prefix(T1_PREFIX) {
            let profile = match TlshProfile::from_raw_length(raw.len()) {
                Some(profile) => profile,
                None => {
                    return Err(TlshError::InvalidDigestLength {
                        actual_length: input.len(),
                    });
                }
            };
            return Self::from_raw_hex_with_profile(raw, profile);
        }

        let profile = match TlshProfile::from_raw_length(input.len()) {
            Some(profile) => profile,
            None => {
                return Err(TlshError::InvalidDigestLength {
                    actual_length: input.len(),
                });
            }
        };
        Self::from_raw_hex_with_profile(input, profile)
    }

    pub fn from_raw_hex(input: &str) -> Result<Self, TlshError> {
        let profile = match TlshProfile::from_raw_length(input.len()) {
            Some(profile) => profile,
            None => {
                return Err(TlshError::InvalidDigestLength {
                    actual_length: input.len(),
                });
            }
        };
        Self::from_raw_hex_with_profile(input, profile)
    }

    pub fn from_raw_hex_with_profile(input: &str, profile: TlshProfile) -> Result<Self, TlshError> {
        if input.len() != profile.raw_length() {
            return Err(TlshError::InvalidDigestLength {
                actual_length: input.len(),
            });
        }

        validate_hex(input.as_bytes())?;

        let bytes = input.as_bytes();
        let mut offset = 0usize;
        let checksum_length = profile.checksum_length();
        let mut checksum = vec![0u8; checksum_length];
        let mut checksum_index = 0usize;
        while checksum_index < checksum_length {
            checksum[checksum_index] = swap_byte(parse_hex_byte(bytes, offset));
            offset += 2;
            checksum_index += 1;
        }

        let lvalue = swap_byte(parse_hex_byte(bytes, offset));
        offset += 2;

        let q_ratios = parse_hex_byte(bytes, offset);
        offset += 2;

        let mut code = vec![0u8; profile.code_size()];
        for idx in 0..profile.code_size() {
            code[profile.code_size() - idx - 1] = parse_hex_byte(bytes, offset + idx * 2);
        }

        Ok(Self {
            profile,
            checksum,
            lvalue,
            q1_ratio: q_ratios >> 4,
            q2_ratio: q_ratios & 0x0F,
            code,
        })
    }

    fn try_diff_with_options(&self, other: &Self, include_length: bool) -> Result<i32, TlshError> {
        if self.profile != other.profile {
            return Err(TlshError::IncompatibleProfiles {
                left: self.profile,
                right: other.profile,
            });
        }

        let mut diff = 0;

        if include_length {
            let ldiff = mod_diff(self.lvalue, other.lvalue, RANGE_LVALUE);
            if ldiff == 1 {
                diff += 1;
            } else if ldiff > 1 {
                diff += ldiff * LENGTH_MULTIPLIER;
            }
        }

        let q1diff = mod_diff(self.q1_ratio, other.q1_ratio, RANGE_QRATIO);
        diff += if q1diff <= 1 {
            q1diff
        } else {
            (q1diff - 1) * 12
        };

        let q2diff = mod_diff(self.q2_ratio, other.q2_ratio, RANGE_QRATIO);
        diff += if q2diff <= 1 {
            q2diff
        } else {
            (q2diff - 1) * 12
        };

        if self.checksum != other.checksum {
            diff += 1;
        }

        Ok(diff + h_distance(&self.code, &other.code))
    }

    fn write_raw_hex(&self, out: &mut String) {
        for checksum in &self.checksum {
            push_hex_byte(out, swap_byte(*checksum));
        }
        push_hex_byte(out, swap_byte(self.lvalue));
        push_hex_byte(out, (self.q1_ratio << 4) | self.q2_ratio);
        let mut index = self.code.len();
        while index > 0 {
            index -= 1;
            push_hex_byte(out, self.code[index]);
        }
    }
}

impl fmt::Display for TlshDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encoded())
    }
}

impl FromStr for TlshDigest {
    type Err = TlshError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_encoded(s)
    }
}

fn validate_hex(bytes: &[u8]) -> Result<(), TlshError> {
    for (idx, byte) in bytes.iter().copied().enumerate() {
        if !byte.is_ascii_hexdigit() {
            return Err(TlshError::InvalidHexCharacter { index: idx, byte });
        }
    }
    Ok(())
}

fn parse_hex_byte(bytes: &[u8], offset: usize) -> u8 {
    (hex_value(bytes[offset]) << 4) | hex_value(bytes[offset + 1])
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'A'..=b'F' => byte - b'A' + 10,
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("hex validated before parsing"),
    }
}

fn push_hex_byte(out: &mut String, byte: u8) {
    out.push(HEX_CHARS[(byte >> 4) as usize] as char);
    out.push(HEX_CHARS[(byte & 0x0F) as usize] as char);
}

fn swap_byte(byte: u8) -> u8 {
    ((byte & 0xF0) >> 4) | ((byte & 0x0F) << 4)
}

fn mod_diff(x: u8, y: u8, range: u16) -> i32 {
    let (dl, dr) = if y > x {
        ((y - x) as i32, x as i32 + range as i32 - y as i32)
    } else {
        ((x - y) as i32, y as i32 + range as i32 - x as i32)
    };
    dl.min(dr)
}

fn h_distance(left: &[u8], right: &[u8]) -> i32 {
    let mut diff = 0;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff += byte_distance(*left_byte, *right_byte);
    }
    diff
}

fn byte_distance(left: u8, right: u8) -> i32 {
    let mut x = left;
    let mut y = right;
    let mut diff = 0;

    for _ in 0..4 {
        diff += pair_distance(x & 0b11, y & 0b11);
        x >>= 2;
        y >>= 2;
    }

    diff
}

fn pair_distance(left: u8, right: u8) -> i32 {
    match left.abs_diff(right) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 6,
        _ => unreachable!("2-bit values only"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_HASH: &str =
        "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";
    const SMALL2_HASH: &str =
        "T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

    fn small() -> TlshDigest {
        TlshDigest::from_encoded(SMALL_HASH).unwrap()
    }

    fn small2() -> TlshDigest {
        TlshDigest::from_encoded(SMALL2_HASH).unwrap()
    }

    #[test]
    fn accessors_and_encoding_helpers_roundtrip() {
        let digest = small();
        assert_eq!(digest.profile(), TlshProfile::standard_t1());
        assert_eq!(digest.checksum().len(), 1);
        assert_eq!(digest.code().len(), digest.profile().code_size());
        assert_eq!(digest.encoded(), SMALL_HASH);
        assert_eq!(digest.encoded_with_version(false), &SMALL_HASH[2..]);
        assert_eq!(digest.raw_hex(), &SMALL_HASH[2..]);
        assert_eq!(digest.to_string(), SMALL_HASH);
        assert_eq!(TlshDigest::from_str(SMALL_HASH).unwrap(), digest);
    }

    #[test]
    fn bucket_value_bounds_are_checked() {
        let digest = small();
        assert!(digest.bucket_value(0).is_some());
        assert!(
            digest
                .bucket_value(digest.profile().effective_buckets() - 1)
                .is_some()
        );
        assert_eq!(
            digest.bucket_value(digest.profile().effective_buckets()),
            None
        );
    }

    #[test]
    fn diff_variants_match_known_values() {
        let left = small();
        let right = small2();
        assert_eq!(left.diff(&right), 221);
        assert_eq!(
            left.diff_no_length(&right),
            left.try_diff_no_length(&right).unwrap()
        );
        assert_eq!(left.try_diff(&right).unwrap(), 221);
    }

    #[test]
    fn incompatible_profile_error_is_returned() {
        let error = small()
            .try_diff(&TlshDigest::from_raw_hex_with_profile(
                "F8F43EA0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD",
                TlshProfile::full_256_3(),
            ).unwrap())
            .unwrap_err();
        assert_eq!(
            error,
            TlshError::IncompatibleProfiles {
                left: TlshProfile::standard_t1(),
                right: TlshProfile::full_256_3(),
            }
        );
    }

    #[test]
    fn parsing_rejects_invalid_hex_and_lengths() {
        assert_eq!(
            TlshDigest::from_encoded("T1ABC").unwrap_err(),
            TlshError::InvalidDigestLength { actual_length: 5 }
        );
        assert_eq!(
            TlshDigest::from_encoded("ABC").unwrap_err(),
            TlshError::InvalidDigestLength { actual_length: 3 }
        );
        assert_eq!(
            TlshDigest::from_raw_hex("GG").unwrap_err(),
            TlshError::InvalidDigestLength { actual_length: 2 }
        );
        assert_eq!(
            TlshDigest::from_raw_hex_with_profile(
                "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
                TlshProfile::standard_t1()
            )
            .unwrap_err(),
            TlshError::InvalidHexCharacter {
                index: 0,
                byte: b'Z'
            }
        );
    }

    #[test]
    fn internal_distance_helpers_cover_all_branches() {
        assert_eq!(mod_diff(0, 15, RANGE_QRATIO), 1);
        assert_eq!(mod_diff(15, 0, RANGE_QRATIO), 1);
        assert_eq!(byte_distance(0b00_01_10_11, 0), 9);
        assert_eq!(pair_distance(0, 0), 0);
        assert_eq!(pair_distance(0, 1), 1);
        assert_eq!(pair_distance(0, 2), 2);
        assert_eq!(pair_distance(0, 3), 6);
        assert_eq!(hex_value(b'0'), 0);
        assert_eq!(hex_value(b'A'), 10);
        assert_eq!(hex_value(b'a'), 10);
        assert_eq!(swap_byte(0xAB), 0xBA);
        let mut out = String::new();
        push_hex_byte(&mut out, 0xAF);
        assert_eq!(out, "AF");
        assert_eq!(parse_hex_byte(b"AF", 0), 0xAF);
        assert!(validate_hex(b"0AaF").is_ok());
    }

    #[test]
    fn digest_accessors_and_raw_parsing_cover_non_prefixed_paths() {
        let digest = small();
        assert!((digest.lvalue() as u16) < RANGE_LVALUE);
        assert!((digest.q1_ratio() as u16) < RANGE_QRATIO);
        assert!((digest.q2_ratio() as u16) < RANGE_QRATIO);

        let raw = &SMALL_HASH[2..];
        assert_eq!(TlshDigest::from_encoded(raw).unwrap(), digest);
        assert_eq!(TlshDigest::from_raw_hex(raw).unwrap(), digest);
    }

    #[test]
    fn digest_parsing_rejects_length_mismatch_for_explicit_profile() {
        let error =
            TlshDigest::from_raw_hex_with_profile("AA", TlshProfile::standard_t1()).unwrap_err();
        assert_eq!(error, TlshError::InvalidDigestLength { actual_length: 2 });
    }

    #[test]
    fn digest_diff_covers_length_and_ratio_penalties() {
        let left = small();
        let mut right = left.clone();
        right.lvalue = ((left.lvalue as u16 + 1) % RANGE_LVALUE) as u8;
        assert_eq!(left.try_diff(&right).unwrap(), 1);

        right.lvalue = ((left.lvalue as u16 + 2) % RANGE_LVALUE) as u8;
        assert_eq!(left.try_diff(&right).unwrap(), 24);

        right = left.clone();
        right.q1_ratio = (left.q1_ratio + 3) % RANGE_QRATIO as u8;
        assert_eq!(left.try_diff(&right).unwrap(), 24);

        right = left.clone();
        right.q2_ratio = (left.q2_ratio + 3) % RANGE_QRATIO as u8;
        assert_eq!(left.try_diff(&right).unwrap(), 24);

        right = left.clone();
        assert_eq!(left.try_diff(&right).unwrap(), 0);
    }

    #[test]
    #[should_panic(expected = "hex validated before parsing")]
    fn hex_value_panics_for_invalid_input() {
        let _ = hex_value(b'Z');
    }

    #[test]
    #[should_panic(expected = "2-bit values only")]
    fn pair_distance_panics_for_invalid_input() {
        let _ = pair_distance(0, 4);
    }
}
