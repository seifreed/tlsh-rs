use tlsh_rs::{TlshBuilder, TlshDigest, TlshProfile, hash_bytes_with_profile};

const STANDARD_SMALL: &str =
    "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";
const STANDARD_SMALL2: &str =
    "T1C6A022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

const PROFILE_128_3_SMALL: &str =
    "F8F43EA0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC";
const PROFILE_128_3_SMALL2: &str =
    "C6305BA022A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

const PROFILE_256_1_SMALL: &str = "F8A0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD";
const PROFILE_256_1_SMALL2: &str = "C6A022AEB23082A308E00303000A30BCB828CAA0CB8B22E300AB8C0380ABB3AC0C0020A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

const PROFILE_256_3_SMALL: &str = "F8F43EA0025A896098CB055024890994B0C2909B9A65F475598139C190185644561C0549584D5F8D5123DB980844DA37E89B8F1C522AB716D2458A071715754E9A55D87AAD";
const PROFILE_256_3_SMALL2: &str = "C6305BA022AEB23082A308E00303000A30BCB828CAA0CB8B22E300AB8C0380ABB3AC0C0020A2E0008CC320C083A3E20AA888022A00000A0AB0088828022A0008A00022F22A";

const SIMPLE_HASH_1: &str =
    "T109F05A198CC69A5A4F0F9380A9EE93F2B927CF42089EA74276DC5F0BB2D34E68114448";
const SIMPLE_HASH_2: &str =
    "T1301124198C869A5A4F0F9380A9AE92F2B9278F42089EA34272885F0FB2D34E6911444C";

#[test]
fn hashes_standard_profile_exactly_like_upstream() {
    let digest = hash_fixture("fixtures/small.txt", TlshProfile::standard_t1());
    assert_eq!(digest.encoded(), STANDARD_SMALL);

    let digest = hash_fixture("fixtures/small2.txt", TlshProfile::standard_t1());
    assert_eq!(digest.encoded(), STANDARD_SMALL2);
}

#[test]
fn hashes_128_3_profile_exactly_like_upstream() {
    let digest = hash_fixture("fixtures/small.txt", TlshProfile::compact_128_3());
    assert_eq!(digest.raw_hex(), PROFILE_128_3_SMALL);
    assert_eq!(
        digest.encoded_with_version(true),
        format!("T1{PROFILE_128_3_SMALL}")
    );

    let digest = hash_fixture("fixtures/small2.txt", TlshProfile::compact_128_3());
    assert_eq!(digest.raw_hex(), PROFILE_128_3_SMALL2);
}

#[test]
fn hashes_256_1_profile_exactly_like_upstream() {
    let digest = hash_fixture("fixtures/small.txt", TlshProfile::full_256_1());
    assert_eq!(digest.raw_hex(), PROFILE_256_1_SMALL);
    assert_eq!(
        digest.encoded_with_version(true),
        format!("T1{PROFILE_256_1_SMALL}")
    );

    let digest = hash_fixture("fixtures/small2.txt", TlshProfile::full_256_1());
    assert_eq!(digest.raw_hex(), PROFILE_256_1_SMALL2);
}

#[test]
fn hashes_256_3_profile_exactly_like_upstream() {
    let digest = hash_fixture("fixtures/small.txt", TlshProfile::full_256_3());
    assert_eq!(digest.raw_hex(), PROFILE_256_3_SMALL);
    assert_eq!(
        digest.encoded_with_version(true),
        format!("T1{PROFILE_256_3_SMALL}")
    );

    let digest = hash_fixture("fixtures/small2.txt", TlshProfile::full_256_3());
    assert_eq!(digest.raw_hex(), PROFILE_256_3_SMALL2);
}

#[test]
fn diff_matches_upstream_for_all_validated_profiles() {
    let standard_left = hash_fixture("fixtures/small.txt", TlshProfile::standard_t1());
    let standard_right = hash_fixture("fixtures/small2.txt", TlshProfile::standard_t1());
    assert_eq!(standard_left.diff(&standard_right), 221);

    let compact_left = hash_fixture("fixtures/small.txt", TlshProfile::compact_128_3());
    let compact_right = hash_fixture("fixtures/small2.txt", TlshProfile::compact_128_3());
    assert_eq!(compact_left.diff(&compact_right), 221);

    let full_1_left = hash_fixture("fixtures/small.txt", TlshProfile::full_256_1());
    let full_1_right = hash_fixture("fixtures/small2.txt", TlshProfile::full_256_1());
    assert_eq!(full_1_left.diff(&full_1_right), 410);

    let full_3_left = hash_fixture("fixtures/small.txt", TlshProfile::full_256_3());
    let full_3_right = hash_fixture("fixtures/small2.txt", TlshProfile::full_256_3());
    assert_eq!(full_3_left.diff(&full_3_right), 410);
}

#[test]
fn roundtrip_supports_prefixed_and_legacy_lengths() {
    let standard = TlshDigest::from_encoded(STANDARD_SMALL).unwrap();
    assert_eq!(standard.encoded(), STANDARD_SMALL);

    let compact = TlshDigest::from_encoded(&format!("T1{PROFILE_128_3_SMALL}")).unwrap();
    assert_eq!(compact.raw_hex(), PROFILE_128_3_SMALL);

    let full = TlshDigest::from_encoded(&format!("T1{PROFILE_256_1_SMALL}")).unwrap();
    assert_eq!(full.raw_hex(), PROFILE_256_1_SMALL);
}

#[test]
fn split_updates_match_single_pass_hash() {
    let data = include_bytes!("../fixtures/small.txt");
    let single = hash_fixture("fixtures/small.txt", TlshProfile::standard_t1());

    let mut builder = TlshBuilder::new();
    builder.update(&data[..17]).unwrap();
    builder.update(&data[17..43]).unwrap();
    builder.update(&data[43..]).unwrap();
    let chunked = builder.finalize().unwrap();

    assert_eq!(single, chunked);
}

#[test]
fn simple_unittest_vectors_match_known_diffs() {
    let left = TlshDigest::from_encoded(SIMPLE_HASH_1).unwrap();
    let right = TlshDigest::from_encoded(SIMPLE_HASH_2).unwrap();

    assert_eq!(left.diff(&left), 0);
    assert_eq!(left.diff(&right), 121);
    assert_eq!(left.diff_no_length(&right), 97);
}

#[test]
fn incompatible_profiles_are_rejected_for_diff() {
    let standard = hash_fixture("fixtures/small.txt", TlshProfile::standard_t1());
    let full = hash_fixture("fixtures/small.txt", TlshProfile::full_256_1());
    assert!(standard.try_diff(&full).is_err());
}

#[test]
fn repeated_bytes_do_not_produce_a_digest() {
    let mut builder = TlshBuilder::new();
    builder.update(&[0u8; 300]).unwrap();
    assert!(builder.finalize().is_err());
}

#[test]
fn helper_hash_bytes_with_profile_matches_builder() {
    let data =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/small.txt"))
            .unwrap();

    let helper = hash_bytes_with_profile(&data, TlshProfile::full_256_3()).unwrap();
    let builder = hash_fixture("fixtures/small.txt", TlshProfile::full_256_3());
    assert_eq!(helper, builder);
}

fn hash_fixture(path: &str, profile: TlshProfile) -> TlshDigest {
    let absolute = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(absolute).unwrap();
    hash_bytes_with_profile(&data, profile).unwrap()
}
