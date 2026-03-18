use wae_crypto::{TotpAlgorithm, TotpSecret, generate_hotp, generate_totp, totp::SecretFormat, verify_hotp, verify_totp};

#[test]
fn test_totp_secret_generate() {
    let secret = TotpSecret::generate(20).unwrap();
    assert_eq!(secret.len(), 20);
    assert!(!secret.is_empty());
}

#[test]
fn test_totp_secret_generate_default() {
    let secret = TotpSecret::generate_default().unwrap();
    assert_eq!(secret.len(), 20);
}

#[test]
fn test_totp_secret_from_bytes() {
    let bytes = b"test_secret_1234567890";
    let secret = TotpSecret::from_bytes(bytes).unwrap();
    assert_eq!(secret.as_bytes(), bytes);
}

#[test]
fn test_totp_secret_from_base32() {
    let base32 = "JBSWY3DPEHPK3PXP";
    let secret = TotpSecret::from_base32(base32).unwrap();
    assert_eq!(secret.as_base32(), base32);
}

#[test]
fn test_totp_secret_format() {
    let bytes = b"test_secret_1234567890";
    let secret = TotpSecret::from_bytes(bytes).unwrap();

    let base32 = secret.format(SecretFormat::Base32);
    assert!(!base32.is_empty());

    let base32_spaced = secret.format(SecretFormat::Base32Spaced);
    assert!(base32_spaced.contains(' '));

    let raw = secret.format(SecretFormat::Raw);
    assert_eq!(raw.len(), bytes.len() * 2);

    let base64 = secret.format(SecretFormat::Base64);
    assert!(!base64.is_empty());
}

#[test]
fn test_totp_secret_display() {
    let bytes = b"test";
    let secret = TotpSecret::from_bytes(bytes).unwrap();
    let display_str = format!("{}", secret);
    assert_eq!(display_str, secret.as_base32());
}

#[test]
fn test_generate_hotp() {
    let secret = b"12345678901234567890";
    let counter = 0;
    let digits = 6;

    let code = generate_hotp(secret, counter, digits, TotpAlgorithm::SHA1).unwrap();
    assert_eq!(code.len(), 6);
}

#[test]
fn test_verify_hotp() {
    let secret = b"12345678901234567890";
    let counter = 0;
    let digits = 6;
    let code = generate_hotp(secret, counter, digits, TotpAlgorithm::SHA1).unwrap();

    let result = verify_hotp(secret, &code, counter, digits, TotpAlgorithm::SHA1).unwrap();
    assert!(result);
}

#[test]
fn test_verify_hotp_wrong_code() {
    let secret = b"12345678901234567890";
    let counter = 0;
    let digits = 6;
    let code = "000000";

    let result = verify_hotp(secret, code, counter, digits, TotpAlgorithm::SHA1).unwrap();
    assert!(!result);
}

#[test]
fn test_generate_totp() {
    let secret = b"12345678901234567890";
    let timestamp = 59;
    let time_step = 30;
    let digits = 6;

    let code = generate_totp(secret, timestamp, time_step, digits, TotpAlgorithm::SHA1).unwrap();
    assert_eq!(code.len(), 6);
}

#[test]
fn test_verify_totp() {
    let secret = b"12345678901234567890";
    let timestamp = 59;
    let time_step = 30;
    let digits = 6;
    let code = generate_totp(secret, timestamp, time_step, digits, TotpAlgorithm::SHA1).unwrap();
    let window = 0;

    let result = verify_totp(secret, &code, timestamp, time_step, digits, TotpAlgorithm::SHA1, window).unwrap();
    assert!(result);
}

#[test]
fn test_verify_totp_with_window() {
    let secret = b"12345678901234567890";
    let timestamp = 59;
    let time_step = 30;
    let digits = 6;
    let code = generate_totp(secret, timestamp, time_step, digits, TotpAlgorithm::SHA1).unwrap();
    let window = 1;

    let result = verify_totp(secret, &code, timestamp, time_step, digits, TotpAlgorithm::SHA1, window).unwrap();
    assert!(result);
}

#[test]
fn test_verify_totp_wrong_code() {
    let secret = b"12345678901234567890";
    let timestamp = 59;
    let time_step = 30;
    let digits = 6;
    let code = "000000";
    let window = 0;

    let result = verify_totp(secret, code, timestamp, time_step, digits, TotpAlgorithm::SHA1, window).unwrap();
    assert!(!result);
}

#[test]
fn test_totp_algorithm_default() {
    let alg = TotpAlgorithm::default();
    assert_eq!(alg, TotpAlgorithm::SHA1);
}

#[test]
fn test_totp_algorithm_clone_copy() {
    let alg = TotpAlgorithm::SHA256;
    let alg2 = alg;
    assert_eq!(alg, alg2);

    let alg3 = alg.clone();
    assert_eq!(alg, alg3);
}

#[test]
fn test_totp_algorithm_debug() {
    let alg = TotpAlgorithm::SHA1;
    let debug_str = format!("{:?}", alg);
    assert!(debug_str.contains("SHA1"));
}

#[test]
fn test_totp_secret_clone() {
    let secret = TotpSecret::generate_default().unwrap();
    let secret2 = secret.clone();
    assert_eq!(secret.as_bytes(), secret2.as_bytes());
}

#[test]
fn test_hotp_different_digits() {
    let secret = b"12345678901234567890";
    let counter = 0;

    for digits in 4..=9 {
        let code = generate_hotp(secret, counter, digits, TotpAlgorithm::SHA1).unwrap();
        assert_eq!(code.len(), digits as usize);
    }
}
