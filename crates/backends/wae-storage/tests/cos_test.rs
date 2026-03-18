use wae_storage::{CosProvider, StorageConfig, StorageProvider, StorageProviderType};

fn hmac_sha1(key: &[u8], msg: &str) -> Vec<u8> {
    use hmac::Mac;
    type HmacSha1 = hmac::Hmac<sha1::Sha1>;
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha1_hex(key: &[u8], msg: &str) -> String {
    hex::encode(hmac_sha1(key, msg))
}

fn cos_encode(s: &str) -> String {
    let encoded = urlencoding::encode(s).to_string();
    ensure_lowercase_hex(&encoded)
}

fn ensure_lowercase_hex(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            result.push(c);
            if let Some(h1) = chars.next() {
                result.push(h1.to_ascii_lowercase());
            }
            if let Some(h2) = chars.next() {
                result.push(h2.to_ascii_lowercase());
            }
        }
        else {
            result.push(c);
        }
    }
    result
}

fn sha1_hex(msg: &str) -> String {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(msg.as_bytes());
    hex::encode(hasher.finalize())
}

#[test]
fn cos_signature_matches_reference_example() {
    let secret_key = "BQYIM75p8x0iWVFSIgqEKwFprpRSVHlz";
    let key_time = "1417773892;1417853898";

    let http_method = "put";
    let http_uri = "/testfile2";
    let http_params = "";
    let http_headers = "host=bucket1-1254000000.cos.ap-beijing.myqcloud.com&x-cos-content-sha1=7b502c3a1f48c8609ae212cdfb639dee39673f5e&x-cos-storage-class=standard";

    let http_string = format!("{}\n{}\n{}\n{}\n", http_method, http_uri, http_params, http_headers);
    let http_string_hash = sha1_hex(&http_string);
    let string_to_sign = format!("sha1\n{}\n{}\n", key_time, http_string_hash);

    let sign_key_hex = hmac_sha1_hex(secret_key.as_bytes(), key_time);
    let signature = hmac_sha1_hex(sign_key_hex.as_bytes(), &string_to_sign);

    assert_eq!(signature, "14e6ebd7955b0c6da532151bf97045e2c5a64e10");
}

#[test]
fn test_sign_url_with_params() {
    let provider = CosProvider;
    let config = StorageConfig {
        provider: StorageProviderType::Cos,
        secret_id: "test_id".to_string(),
        secret_key: "test_key".to_string(),
        bucket: "test_bucket".to_string(),
        region: "ap-nanjing".to_string(),
        endpoint: None,
        cdn_url: None,
    };

    let url = provider.sign_url("/test.png?imageMogr2/thumbnail/400x400&name=hello world", &config).unwrap().to_string();
    println!("Generated URL: {}", url);

    assert!(url.contains("q-url-param-list=imagemogr2%252fthumbnail%252f400x400%3bname"));
    assert!(url.contains("q-sign-algorithm=sha1"));
    assert!(url.find("q-signature=").unwrap() < url.find("imageMogr2/thumbnail/400x400").unwrap());
    assert!(url.contains("&imageMogr2/thumbnail/400x400"));
    assert!(!url.contains("imageMogr2/thumbnail/400x400="));
    assert!(url.contains("name=hello%20world"));
}
