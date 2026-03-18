use wae_types::*;

#[test]
fn test_format_slug() {
    assert_eq!(format_slug("Hello World"), "hello-world");
    assert_eq!(format_slug("Test 123"), "test-123");
    assert_eq!(format_slug(""), "");
    assert_eq!(format_slug("Already-Slugged"), "already-slugged");
}

#[test]
fn test_truncate_str() {
    assert_eq!(truncate_str("hello", 10), "hello");
    assert_eq!(truncate_str("hello world", 5), "hello...");
    assert_eq!(truncate_str("", 5), "");
    assert_eq!(truncate_str("test", 4), "test");
}

#[test]
fn test_hex_encode() {
    assert_eq!(hex_encode(&[0x00, 0x01, 0x02]), "000102");
    assert_eq!(hex_encode(&[0xff, 0xee, 0xdd]), "ffeedd");
    assert_eq!(hex_encode(&[]), "");
    assert_eq!(hex_encode(&[0x12, 0x34, 0x56]), "123456");
}

#[test]
fn test_hex_decode() {
    let expected1: Vec<u8> = vec![0x00, 0x01, 0x02];
    assert_eq!(hex_decode("000102").unwrap(), expected1);

    let expected2: Vec<u8> = vec![0xff, 0xee, 0xdd];
    assert_eq!(hex_decode("ffeedd").unwrap(), expected2);

    assert_eq!(hex_decode("").unwrap(), Vec::<u8>::new());

    let expected3: Vec<u8> = vec![0x12, 0x34, 0x56];
    assert_eq!(hex_decode("123456").unwrap(), expected3);

    assert!(hex_decode("1").is_err());
    assert!(hex_decode("gg").is_err());
}

#[test]
fn test_url_encode() {
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(url_encode("test=123"), "test%3D123");
    assert_eq!(url_encode("safe-chars._~"), "safe-chars._~");
}

#[test]
fn test_url_decode() {
    assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
    assert_eq!(url_decode("test%3D123").unwrap(), "test=123");
    assert_eq!(url_decode("hello+world").unwrap(), "hello world");
    assert_eq!(url_decode("safe-chars._~").unwrap(), "safe-chars._~");

    assert!(url_decode("%GG").is_err());
}

#[test]
fn test_url_encode_decode_roundtrip() {
    let test_str = "Hello 世界! @#$%^&*()";
    let encoded = url_encode(test_str);
    let decoded = url_decode(&encoded).unwrap();
    assert_eq!(decoded, test_str);
}

#[test]
fn test_hex_encode_decode_roundtrip() {
    let test_bytes = vec![0x00, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xff];
    let encoded = hex_encode(&test_bytes);
    let decoded = hex_decode(&encoded).unwrap();
    assert_eq!(decoded, test_bytes);
}
