use hex::ToHex;
use wae_crypto::{HashAlgorithm, hash};

#[test]
fn test_hash_sha1() {
    let data = b"hello world";
    let result = hash(HashAlgorithm::SHA1, data).unwrap();
    let hex = result.encode_hex::<String>();
    assert_eq!(hex, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
}

#[test]
fn test_hash_sha256() {
    let data = b"hello world";
    let result = hash(HashAlgorithm::SHA256, data).unwrap();
    let hex = result.encode_hex::<String>();
    assert_eq!(hex, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
}

#[test]
fn test_hash_sha384() {
    let data = b"hello world";
    let result = hash(HashAlgorithm::SHA384, data).unwrap();
    assert_eq!(result.len(), 48);
}

#[test]
fn test_hash_sha512() {
    let data = b"hello world";
    let result = hash(HashAlgorithm::SHA512, data).unwrap();
    assert_eq!(result.len(), 64);
}

#[test]
fn test_hash_empty_input() {
    let data = b"";

    let sha1_result = hash(HashAlgorithm::SHA1, data).unwrap();
    assert_eq!(sha1_result.len(), 20);

    let sha256_result = hash(HashAlgorithm::SHA256, data).unwrap();
    assert_eq!(sha256_result.len(), 32);

    let sha384_result = hash(HashAlgorithm::SHA384, data).unwrap();
    assert_eq!(sha384_result.len(), 48);

    let sha512_result = hash(HashAlgorithm::SHA512, data).unwrap();
    assert_eq!(sha512_result.len(), 64);
}

#[test]
fn test_hash_algorithm_clone_copy() {
    let alg = HashAlgorithm::SHA256;
    let alg2 = alg;
    assert_eq!(alg, alg2);

    let alg3 = alg.clone();
    assert_eq!(alg, alg3);
}

#[test]
fn test_hash_algorithm_debug() {
    let alg = HashAlgorithm::SHA1;
    let debug_str = format!("{:?}", alg);
    assert!(debug_str.contains("SHA1"));
}
