use serde::{Deserialize, Serialize};
use wae_authentication::jwt::codec::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestClaims {
    sub: String,
    exp: i64,
    iat: i64,
}

#[test]
fn test_encode_decode_jwt() {
    let secret = b"test-secret";
    let claims = TestClaims {
        sub: "test-user".to_string(),
        exp: chrono::Utc::now().timestamp() + 3600,
        iat: chrono::Utc::now().timestamp(),
    };

    let header = JwtHeader::new("HS256");
    let token = encode_jwt(&header, &claims, secret).unwrap();
    let decoded: TestClaims = decode_jwt(&token, secret, true).unwrap();

    assert_eq!(claims, decoded);
}
