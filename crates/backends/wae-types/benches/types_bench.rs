use criterion::{Criterion, criterion_group, criterion_main};
use std::{collections::HashMap, hint::black_box};
use wae_types::*;

fn bench_value_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_creation");

    group.bench_function("null", |b| b.iter(|| Value::null()));

    group.bench_function("bool_true", |b| b.iter(|| Value::bool(black_box(true))));

    group.bench_function("integer", |b| b.iter(|| Value::integer(black_box(42))));

    group.bench_function("float", |b| b.iter(|| Value::float(black_box(3.14159))));

    group.bench_function("string", |b| b.iter(|| Value::string(black_box("Hello, World!"))));

    group.bench_function("array_empty", |b| b.iter(|| Value::array(black_box(Vec::new()))));

    group.bench_function("object_empty", |b| b.iter(|| Value::object(black_box(HashMap::new()))));

    group.finish();
}

fn bench_value_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_conversions");

    let bool_val = Value::bool(true);
    group.bench_function("as_bool", |b| b.iter(|| black_box(&bool_val).as_bool()));

    let int_val = Value::integer(42);
    group.bench_function("as_integer", |b| b.iter(|| black_box(&int_val).as_integer()));

    let float_val = Value::float(3.14159);
    group.bench_function("as_float", |b| b.iter(|| black_box(&float_val).as_float()));

    let string_val = Value::string("Hello, World!");
    group.bench_function("as_str", |b| b.iter(|| black_box(&string_val).as_str()));

    let array_val = Value::array(vec![Value::integer(1), Value::integer(2), Value::integer(3)]);
    group.bench_function("as_array", |b| b.iter(|| black_box(&array_val).as_array()));

    let mut map = HashMap::new();
    map.insert("key1".to_string(), Value::string("value1"));
    map.insert("key2".to_string(), Value::integer(42));
    let object_val = Value::object(map);
    group.bench_function("as_object", |b| b.iter(|| black_box(&object_val).as_object()));

    group.finish();
}

fn bench_macros(c: &mut Criterion) {
    let mut group = c.benchmark_group("macros");

    group.bench_function("object_macro", |b| {
        b.iter(|| {
            object! {
                "name" => "Test",
                "age" => 42,
                "active" => true
            }
        })
    });

    group.bench_function("array_macro", |b| b.iter(|| array![1, 2, 3, 4, 5]));

    group.finish();
}

fn bench_util_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("util_functions");

    let test_str = "Hello World! This is a Test String.";
    group.bench_function("format_slug", |b| b.iter(|| format_slug(black_box(test_str))));

    let long_str = "This is a very long string that needs to be truncated for logging purposes.";
    group.bench_function("truncate_str", |b| b.iter(|| truncate_str(black_box(long_str), black_box(20))));

    let bytes = b"Hello, World!";
    group.bench_function("hex_encode", |b| b.iter(|| hex_encode(black_box(bytes))));

    let hex_str = "48656c6c6f2c20576f726c6421";
    group.bench_function("hex_decode", |b| b.iter(|| hex_decode(black_box(hex_str))));

    let url_str = "Hello World! How are you?";
    group.bench_function("url_encode", |b| b.iter(|| url_encode(black_box(url_str))));

    let encoded_url = "Hello%20World%21%20How%20are%20you%3F";
    group.bench_function("url_decode", |b| b.iter(|| url_decode(black_box(encoded_url))));

    group.finish();
}

criterion_group!(benches, bench_value_creation, bench_value_conversions, bench_macros, bench_util_functions);
criterion_main!(benches);
