use hex::ToHex;
use wae_crypto::{HmacAlgorithm, hmac_sign, hmac_verify};

#[test]
fn test_hmac_sha1() {
    let secret = b"test_secret";
    let data = b"hello world";
    let signature = hmac_sign(HmacAlgorithm::SHA1, secret, data).unwrap();
    assert_eq!(signature.len(), 20);
}

#[test]
fn test_hmac_sha256() {
    let secret = b"test_secret";
    let data = b"hello world";
    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    assert_eq!(signature.len(), 32);
}

#[test]
fn test_hmac_sha384() {
    let secret = b"test_secret";
    let data = b"hello world";
    let signature = hmac_sign(HmacAlgorithm::SHA384, secret, data).unwrap();
    let hex = signature.encode_hex::<String>();

    assert_eq!(hex.len(), 96);
}

#[test]
fn test_hmac_sha512() {
    let secret = b"test_secret";
    let data = b"hello world";
    let signature = hmac_sign(HmacAlgorithm::SHA512, secret, data).unwrap();
    let hex = signature.encode_hex::<String>();

    assert_eq!(hex.len(), 128);
}

#[test]
fn test_hmac_verify_correct() {
    let secret = b"test_secret";
    let data = b"hello world";

    let algorithms = vec![HmacAlgorithm::SHA1, HmacAlgorithm::SHA256, HmacAlgorithm::SHA384, HmacAlgorithm::SHA512];

    for alg in algorithms {
        let signature = hmac_sign(alg, secret, data).unwrap();
        let result = hmac_verify(alg, secret, data, &signature).unwrap();
        assert!(result);
    }
}

#[test]
fn test_hmac_verify_incorrect_signature() {
    let secret = b"test_secret";
    let data = b"hello world";
    let wrong_data = b"wrong data";

    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    let result = hmac_verify(HmacAlgorithm::SHA256, secret, wrong_data, &signature);
    assert!(result.is_err());
}

#[test]
fn test_hmac_verify_incorrect_secret() {
    let secret = b"test_secret";
    let wrong_secret = b"wrong_secret";
    let data = b"hello world";

    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    let result = hmac_verify(HmacAlgorithm::SHA256, wrong_secret, data, &signature);
    assert!(result.is_err());
}

#[test]
fn test_hmac_empty_input() {
    let secret = b"test_secret";
    let data = b"";

    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    let result = hmac_verify(HmacAlgorithm::SHA256, secret, data, &signature).unwrap();
    assert!(result);
}

#[test]
fn test_hmac_empty_secret() {
    let secret = b"";
    let data = b"hello world";

    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    let result = hmac_verify(HmacAlgorithm::SHA256, secret, data, &signature).unwrap();
    assert!(result);
}

#[test]
fn test_hmac_algorithm_clone_copy() {
    let alg = HmacAlgorithm::SHA256;
    let alg2 = alg;
    assert_eq!(alg, alg2);

    let alg3 = alg.clone();
    assert_eq!(alg, alg3);
}

#[test]
fn test_hmac_algorithm_debug() {
    let alg = HmacAlgorithm::SHA1;
    let debug_str = format!("{:?}", alg);
    assert!(debug_str.contains("SHA1"));
}
