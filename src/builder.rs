use crate::digest::TlshDigest;
use crate::error::TlshError;
use crate::internal::constants::{
    BUCKETS, MAX_DATA_LENGTH, MIN_CONSERVATIVE_DATA_LENGTH, MIN_DATA_LENGTH, PEARSON_TABLE,
    SLIDING_WINDOW_SIZE, TOP_VALUE,
};
use crate::internal::quartile::find_quartiles;
use crate::profile::TlshProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TlshOptions {
    pub conservative: bool,
}

#[derive(Debug, Clone)]
pub struct TlshBuilder {
    profile: TlshProfile,
    buckets: [u32; BUCKETS],
    slide_window: [u8; SLIDING_WINDOW_SIZE],
    data_len: u64,
    checksum: [u8; 3],
}

impl Default for TlshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TlshBuilder {
    pub fn new() -> Self {
        Self::with_profile(TlshProfile::standard_t1())
    }

    pub fn with_profile(profile: TlshProfile) -> Self {
        Self {
            profile,
            buckets: [0; BUCKETS],
            slide_window: [0; SLIDING_WINDOW_SIZE],
            data_len: 0,
            checksum: [0; 3],
        }
    }

    pub fn profile(&self) -> TlshProfile {
        self.profile
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), TlshError> {
        if self.data_len + data.len() as u64 > MAX_DATA_LENGTH {
            return Err(TlshError::DataTooLong);
        }

        let rng_size = SLIDING_WINDOW_SIZE;
        let mut j = (self.data_len as usize) % rng_size;
        let mut j_1 = (j + rng_size - 1) % rng_size;
        let mut j_2 = (j + rng_size - 2) % rng_size;
        let mut j_3 = (j + rng_size - 3) % rng_size;
        let mut j_4 = (j + rng_size - 4) % rng_size;
        let mut fed_len = self.data_len;

        for &byte in data {
            self.slide_window[j] = byte;

            if fed_len >= 4 {
                self.update_checksum(j, j_1);
                self.update_buckets(j, j_1, j_2, j_3, j_4);
            }

            let j_tmp = j_4;
            j_4 = j_3;
            j_3 = j_2;
            j_2 = j_1;
            j_1 = j;
            j = j_tmp;
            fed_len += 1;
        }

        self.data_len += data.len() as u64;
        Ok(())
    }

    pub fn reset(&mut self) {
        *self = Self::with_profile(self.profile);
    }

    pub fn len(&self) -> u64 {
        self.data_len
    }

    pub fn is_empty(&self) -> bool {
        self.data_len == 0
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid_with_options(TlshOptions::default())
    }

    pub fn is_valid_with_options(&self, options: TlshOptions) -> bool {
        let min_length = if options.conservative {
            MIN_CONSERVATIVE_DATA_LENGTH
        } else {
            MIN_DATA_LENGTH
        };

        if self.data_len < min_length as u64 {
            return false;
        }

        let nonzero = self.buckets[..self.profile.effective_buckets()]
            .iter()
            .filter(|&&count| count > 0)
            .count();

        nonzero > (self.profile.effective_buckets() / 2)
    }

    pub fn finalize(&self) -> Result<TlshDigest, TlshError> {
        self.finalize_with_options(TlshOptions::default())
    }

    pub fn finalize_with_options(&self, options: TlshOptions) -> Result<TlshDigest, TlshError> {
        let min_length = if options.conservative {
            MIN_CONSERVATIVE_DATA_LENGTH
        } else {
            MIN_DATA_LENGTH
        };

        if self.data_len < min_length as u64 {
            return Err(TlshError::TooShort {
                min_length,
                actual_length: self.data_len as usize,
            });
        }

        let effective = &self.buckets[..self.profile.effective_buckets()];
        let nonzero = effective.iter().filter(|&&count| count > 0).count();
        if nonzero <= self.profile.effective_buckets() / 2 {
            return Err(TlshError::InsufficientVariance);
        }

        let (q1, q2, q3) = find_quartiles(effective);

        let mut code = vec![0u8; self.profile.code_size()];
        for (idx, chunk) in effective.chunks_exact(4).enumerate() {
            let mut value = 0u8;
            for (offset, bucket) in chunk.iter().copied().enumerate() {
                if q3 < bucket {
                    value += 3 << (offset * 2);
                } else if q2 < bucket {
                    value += 2 << (offset * 2);
                } else if q1 < bucket {
                    value += 1 << (offset * 2);
                }
            }
            code[idx] = value;
        }

        let lvalue = capture_length(self.data_len)?;
        let q1_ratio = (((q1 as u64) * 100) / (q3 as u64) % 16) as u8;
        let q2_ratio = (((q2 as u64) * 100) / (q3 as u64) % 16) as u8;

        Ok(TlshDigest::new(
            self.profile,
            self.checksum[..self.profile.checksum_length()].to_vec(),
            lvalue,
            q1_ratio,
            q2_ratio,
            code,
        ))
    }

    fn update_checksum(&mut self, j: usize, j_1: usize) {
        let current = self.slide_window;
        self.checksum[0] = b_mapping(current[j], current[j_1], self.checksum[0], 0);
        for idx in 1..self.profile.checksum_length() {
            self.checksum[idx] = b_mapping(
                current[j],
                current[j_1],
                self.checksum[idx],
                self.checksum[idx - 1],
            );
        }
    }

    fn update_buckets(&mut self, j: usize, j_1: usize, j_2: usize, j_3: usize, j_4: usize) {
        let window = self.slide_window;

        let mut bucket = b_mapping(window[j], window[j_1], window[j_2], 2);
        self.buckets[bucket as usize] += 1;

        bucket = b_mapping(window[j], window[j_1], window[j_3], 3);
        self.buckets[bucket as usize] += 1;

        bucket = b_mapping(window[j], window[j_2], window[j_3], 5);
        self.buckets[bucket as usize] += 1;

        bucket = b_mapping(window[j], window[j_2], window[j_4], 7);
        self.buckets[bucket as usize] += 1;

        bucket = b_mapping(window[j], window[j_1], window[j_4], 11);
        self.buckets[bucket as usize] += 1;

        bucket = b_mapping(window[j], window[j_3], window[j_4], 13);
        self.buckets[bucket as usize] += 1;
    }
}

pub fn hash_bytes(data: &[u8]) -> Result<TlshDigest, TlshError> {
    hash_bytes_with_profile(data, TlshProfile::standard_t1())
}

pub fn hash_bytes_with_profile(data: &[u8], profile: TlshProfile) -> Result<TlshDigest, TlshError> {
    let mut builder = TlshBuilder::with_profile(profile);
    builder.update(data)?;
    builder.finalize()
}

fn b_mapping(i: u8, j: u8, k: u8, salt: u8) -> u8 {
    let mut h = PEARSON_TABLE[salt as usize];
    h = PEARSON_TABLE[(h ^ i) as usize];
    h = PEARSON_TABLE[(h ^ j) as usize];
    PEARSON_TABLE[(h ^ k) as usize]
}

fn capture_length(len: u64) -> Result<u8, TlshError> {
    let mut bottom = 0usize;
    let mut top = TOP_VALUE.len();
    let mut idx = top >> 1;

    while idx < TOP_VALUE.len() {
        if idx == 0 {
            return Ok(0);
        }
        if len <= TOP_VALUE[idx] as u64 && len > TOP_VALUE[idx - 1] as u64 {
            return Ok(idx as u8);
        }
        if len < TOP_VALUE[idx] as u64 {
            top = idx - 1;
        } else {
            bottom = idx + 1;
        }
        idx = (bottom + top) >> 1;
    }

    Err(TlshError::DataTooLong)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &[u8] = include_bytes!("../fixtures/small.txt");

    #[test]
    fn builder_default_state_and_reset_are_consistent() {
        let mut builder = TlshBuilder::new();
        assert_eq!(builder.profile(), TlshProfile::standard_t1());
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);

        builder.update(b"abcdef").unwrap();
        assert!(!builder.is_empty());
        assert_eq!(builder.len(), 6);

        builder.reset();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
        assert_eq!(builder.profile(), TlshProfile::standard_t1());
    }

    #[test]
    fn builder_validity_checks_cover_short_and_conservative_paths() {
        let mut short = TlshBuilder::new();
        short.update(&SMALL[..40]).unwrap();
        assert!(!short.is_valid());
        assert!(!short.is_valid_with_options(TlshOptions { conservative: true }));

        let mut full = TlshBuilder::new();
        full.update(SMALL).unwrap();
        assert!(full.is_valid());
        assert!(full.finalize().is_ok());
    }

    #[test]
    fn builder_finalize_with_conservative_option_rejects_small_input() {
        let mut builder = TlshBuilder::new();
        builder.update(SMALL).unwrap();
        let error = builder
            .finalize_with_options(TlshOptions { conservative: true })
            .unwrap_err();
        assert_eq!(
            error,
            TlshError::TooShort {
                min_length: MIN_CONSERVATIVE_DATA_LENGTH,
                actual_length: SMALL.len(),
            }
        );
    }

    #[test]
    fn builder_detects_data_too_long_before_processing() {
        let mut builder = TlshBuilder::new();
        builder.data_len = MAX_DATA_LENGTH;
        let error = builder.update(&[1]).unwrap_err();
        assert_eq!(error, TlshError::DataTooLong);
    }

    #[test]
    fn helper_hash_functions_match_known_digest() {
        let digest = hash_bytes(SMALL).unwrap();
        assert_eq!(
            digest.encoded(),
            "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC"
        );
        let digest_with_profile =
            hash_bytes_with_profile(SMALL, TlshProfile::standard_t1()).unwrap();
        assert_eq!(digest, digest_with_profile);
    }

    #[test]
    fn capture_length_covers_boundary_cases() {
        assert_eq!(capture_length(0).unwrap(), 0);
        assert_eq!(capture_length(TOP_VALUE[1] as u64).unwrap(), 1);
        assert_eq!(
            capture_length(TOP_VALUE[TOP_VALUE.len() - 1] as u64 + 1).unwrap_err(),
            TlshError::DataTooLong
        );
    }

    #[test]
    fn b_mapping_is_stable() {
        assert_eq!(b_mapping(1, 2, 3, 4), b_mapping(1, 2, 3, 4));
    }

    #[test]
    fn builder_default_impl_matches_new() {
        let builder = TlshBuilder::default();
        assert_eq!(builder.profile(), TlshProfile::standard_t1());
        assert!(builder.is_empty());
    }

    #[test]
    fn builder_supports_three_byte_checksum_profile() {
        let mut builder = TlshBuilder::with_profile(TlshProfile::compact_128_3());
        builder.update(SMALL).unwrap();
        let digest = builder.finalize().unwrap();
        assert_eq!(digest.profile(), TlshProfile::compact_128_3());
        assert_eq!(digest.checksum().len(), 3);
    }

    #[test]
    fn builder_finalize_detects_insufficient_variance_paths() {
        let mut builder = TlshBuilder::new();
        builder.data_len = MIN_DATA_LENGTH as u64;
        for count in builder
            .buckets
            .iter_mut()
            .take(builder.profile.effective_buckets() / 2)
        {
            *count = 1;
        }
        assert_eq!(
            builder.finalize().unwrap_err(),
            TlshError::InsufficientVariance
        );
    }

    #[test]
    fn builder_quantization_covers_q1_bucket_branch() {
        let mut builder = TlshBuilder::new();
        builder.data_len = MIN_DATA_LENGTH as u64;
        let effective = builder.profile.effective_buckets();
        for count in builder.buckets.iter_mut().take(effective) {
            *count = 4;
        }
        builder.buckets[0] = 1;
        builder.buckets[1] = 2;
        builder.buckets[2] = 3;
        builder.buckets[4..35].fill(1);
        builder.buckets[35..66].fill(2);
        builder.buckets[66..97].fill(3);
        let digest = builder.finalize().unwrap();
        assert_eq!(digest.code()[0], 0b1110_0100);
    }
}
