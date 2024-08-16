use base64::{Engine as _, engine::general_purpose};
use base64::alphabet::URL_SAFE;
use base64::engine::GeneralPurpose;
use serde_json::{Value, from_str};





fn extract_jwt_payload(jwt_token: &str) -> Option<Value> {
    let payload_base64 = jwt_token.split('.').nth(1)?;
    let payload_bytes = GeneralPurpose::new(
        &URL_SAFE,
        general_purpose::NO_PAD)
        .decode(payload_base64).unwrap();
    let payload_str = std::str::from_utf8(&payload_bytes).unwrap();
    from_str(payload_str).unwrap()
}
