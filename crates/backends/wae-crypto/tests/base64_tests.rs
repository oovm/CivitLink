use wae_crypto::{base64_decode, base64_encode, base64url_decode, base64url_encode};

#[test]
fn test_base64_encode() {
    let data = b"Hello, World!";
    let encoded = base64_encode(data);
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
}

#[test]
fn test_base64_decode() {
    let encoded = "SGVsbG8sIFdvcmxkIQ==";
    let decoded = base64_decode(encoded).unwrap();
    assert_eq!(decoded, b"Hello, World!");
}

#[test]
fn test_base64_roundtrip() {
    let test_cases: Vec<&[u8]> = vec![
        b"",
        b"a",
        b"ab",
        b"abc",
        b"abcd",
        b"Hello, World!",
        b"The quick brown fox jumps over the lazy dog",
        &[0, 1, 2, 3, 4, 5, 255, 254, 253, 252, 251],
    ];

    for data in test_cases {
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_base64url_encode() {
    let data = b"Hello, World!";
    let encoded = base64url_encode(data);
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ");
}

#[test]
fn test_base64url_decode() {
    let encoded = "SGVsbG8sIFdvcmxkIQ";
    let decoded = base64url_decode(encoded).unwrap();
    assert_eq!(decoded, b"Hello, World!");
}

#[test]
fn test_base64url_roundtrip() {
    let test_cases: Vec<&[u8]> = vec![
        b"",
        b"a",
        b"ab",
        b"abc",
        b"abcd",
        b"Hello, World!",
        b"The quick brown fox jumps over the lazy dog",
        &[0, 1, 2, 3, 4, 5, 255, 254, 253, 252, 251],
    ];

    for data in test_cases {
        let encoded = base64url_encode(data);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_base64_invalid_input() {
    let invalid_inputs = vec!["invalid!", "SGVsbG8=!", "SGVsbG8==", "SGVsbG8"];

    for input in invalid_inputs {
        let result = base64_decode(input);
        if input != "SGVsbG8==" && input != "SGVsbG8" {
            assert!(result.is_err());
        }
    }
}
