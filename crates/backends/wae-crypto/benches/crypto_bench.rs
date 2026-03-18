use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wae_crypto::*;

fn bench_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash");

    let data = b"Hello, World! This is a test string for benchmarking.";

    group.bench_function("sha1", |b| b.iter(|| hash(black_box(HashAlgorithm::SHA1), black_box(data))));

    group.bench_function("sha256", |b| b.iter(|| hash(black_box(HashAlgorithm::SHA256), black_box(data))));

    group.bench_function("sha384", |b| b.iter(|| hash(black_box(HashAlgorithm::SHA384), black_box(data))));

    group.bench_function("sha512", |b| b.iter(|| hash(black_box(HashAlgorithm::SHA512), black_box(data))));

    group.finish();
}

fn bench_hmac(c: &mut Criterion) {
    let mut group = c.benchmark_group("hmac");

    let secret = b"test-secret-key-12345";
    let data = b"Hello, World! This is a test string for benchmarking.";

    group.bench_function("sign_sha1", |b| {
        b.iter(|| hmac_sign(black_box(HmacAlgorithm::SHA1), black_box(secret), black_box(data)))
    });

    group.bench_function("sign_sha256", |b| {
        b.iter(|| hmac_sign(black_box(HmacAlgorithm::SHA256), black_box(secret), black_box(data)))
    });

    let signature = hmac_sign(HmacAlgorithm::SHA256, secret, data).unwrap();

    group.bench_function("verify_sha256", |b| {
        b.iter(|| hmac_verify(black_box(HmacAlgorithm::SHA256), black_box(secret), black_box(data), black_box(&signature)))
    });

    group.finish();
}

fn bench_base64(c: &mut Criterion) {
    let mut group = c.benchmark_group("base64");

    let data = b"Hello, World! This is a test string for benchmarking base64 encoding and decoding.";

    group.bench_function("encode", |b| b.iter(|| base64_encode(black_box(data))));

    let encoded = base64_encode(data);

    group.bench_function("decode", |b| b.iter(|| base64_decode(black_box(&encoded))));

    group.bench_function("url_encode", |b| b.iter(|| base64url_encode(black_box(data))));

    let url_encoded = base64url_encode(data);

    group.bench_function("url_decode", |b| b.iter(|| base64url_decode(black_box(&url_encoded))));

    group.finish();
}

fn bench_password(c: &mut Criterion) {
    let mut group = c.benchmark_group("password");

    let password = "test-password-12345";
    let hasher = PasswordHasher::default();

    group.sample_size(10);
    group.bench_function("hash_bcrypt", |b| b.iter(|| hasher.hash_password(black_box(password))));

    let hashed = hasher.hash_password(password).unwrap();

    group.bench_function("verify_bcrypt", |b| b.iter(|| hasher.verify_password(black_box(password), black_box(&hashed))));

    group.finish();
}

criterion_group!(benches, bench_hash, bench_hmac, bench_base64, bench_password);
criterion_main!(benches);
