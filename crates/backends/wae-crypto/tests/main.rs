use wae_crypto::*;

#[test]
fn test_base64() {
    let data = b"hello world";
    let encoded = base64_encode(data);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(data, decoded.as_slice());

    let url_encoded = base64url_encode(data);
    let url_decoded = base64url_decode(&url_encoded).unwrap();
    assert_eq!(data, url_decoded.as_slice());
}

#[test]
fn test_hash() {
    let data = b"test data";
    let sha1 = hash(HashAlgorithm::SHA1, data).unwrap();
    assert_eq!(sha1.len(), 20);

    let sha256 = hash(HashAlgorithm::SHA256, data).unwrap();
    assert_eq!(sha256.len(), 32);

    let sha384 = hash(HashAlgorithm::SHA384, data).unwrap();
    assert_eq!(sha384.len(), 48);

    let sha512 = hash(HashAlgorithm::SHA512, data).unwrap();
    assert_eq!(sha512.len(), 64);
}

#[test]
fn test_hmac() {
    let secret = b"test-secret";
    let data = b"test data";

    let sign1 = hmac_sign(HmacAlgorithm::SHA1, secret, data).unwrap();
    assert!(hmac_verify(HmacAlgorithm::SHA1, secret, data, &sign1).unwrap());

    let sign256 = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();
    assert!(hmac_verify(HmacAlgorithm::SHA256, secret, data, &sign256).unwrap());
}

#[test]
fn test_password() {
    let hasher = PasswordHasher::default();
    let password = "test-password-123";
    let hash = hasher.hash_password(password).unwrap();
    assert!(hasher.verify_password(password, &hash).unwrap());
    assert!(!hasher.verify_password("wrong-password", &hash).unwrap());
}

#[test]
fn test_password_argon2() {
    let config = PasswordHasherConfig { algorithm: PasswordAlgorithm::Argon2, ..PasswordHasherConfig::default() };
    let hasher = PasswordHasher::new(config);
    let password = "test-password-123";
    let hash = hasher.hash_password(password).unwrap();
    assert!(hasher.verify_password(password, &hash).unwrap());
    assert!(!hasher.verify_password("wrong-password", &hash).unwrap());
}

#[test]
fn test_totp_secret() {
    let secret = TotpSecret::generate_default().unwrap();
    assert_eq!(secret.len(), 20);

    let base32 = secret.as_base32();
    let from_base32 = TotpSecret::from_base32(base32).unwrap();
    assert_eq!(secret.as_bytes(), from_base32.as_bytes());
}

#[test]
fn test_hotp() {
    let secret = TotpSecret::generate_default().unwrap();
    let code = generate_hotp(secret.as_bytes(), 0, 6, TotpAlgorithm::SHA1).unwrap();
    assert_eq!(code.len(), 6);
    assert!(verify_hotp(secret.as_bytes(), &code, 0, 6, TotpAlgorithm::SHA1).unwrap());
}
