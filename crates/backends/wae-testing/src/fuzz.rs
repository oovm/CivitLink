//! Fuzz testing utilities for WAE.
//!
//! This module provides utilities for generating safe test inputs and integrating
//! with fuzz testing frameworks like `cargo-fuzz` and `honggfuzz`.

pub use arbitrary::{Arbitrary, Unstructured};

/// A trait for types that can generate safe, constrained values for fuzz testing.
///
/// Unlike `Arbitrary`, which generates arbitrary values, `SafeArbitrary` generates
/// values that are within safe or valid ranges, making it suitable for testing
/// functions that expect valid inputs.
pub trait SafeArbitrary<'a>: Sized {
    /// Generate a safe, constrained value from the given unstructured data.
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self>;
}

impl<'a> SafeArbitrary<'a> for String {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(0..=1024)?;
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(u.int_in_range(0x20..=0x7E)?);
        }
        Ok(String::from_utf8(bytes).unwrap())
    }
}

impl<'a, T: SafeArbitrary<'a>> SafeArbitrary<'a> for Vec<T> {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(0..=100)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::safe_arbitrary(u)?);
        }
        Ok(vec)
    }
}

impl<'a, T: SafeArbitrary<'a>> SafeArbitrary<'a> for Option<T> {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.arbitrary()? { Ok(Some(T::safe_arbitrary(u)?)) } else { Ok(None) }
    }
}

impl<'a> SafeArbitrary<'a> for i32 {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.int_in_range(i32::MIN..=i32::MAX)
    }
}

impl<'a> SafeArbitrary<'a> for u32 {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.int_in_range(u32::MIN..=u32::MAX)
    }
}

impl<'a> SafeArbitrary<'a> for i64 {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.int_in_range(i64::MIN..=i64::MAX)
    }
}

impl<'a> SafeArbitrary<'a> for u64 {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.int_in_range(u64::MIN..=u64::MAX)
    }
}

impl<'a> SafeArbitrary<'a> for f64 {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let bits: u64 = u.arbitrary()?;
        let f = f64::from_bits(bits);
        if f.is_finite() { Ok(f) } else { Ok(0.0) }
    }
}

impl<'a> SafeArbitrary<'a> for bool {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary()
    }
}

impl<'a, T: SafeArbitrary<'a>, E: SafeArbitrary<'a>> SafeArbitrary<'a> for Result<T, E> {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.arbitrary()? { Ok(Ok(T::safe_arbitrary(u)?)) } else { Ok(Err(E::safe_arbitrary(u)?)) }
    }
}

impl<'a, const N: usize, T: SafeArbitrary<'a> + Default + Copy> SafeArbitrary<'a> for [T; N] {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut arr = [T::default(); N];
        for item in arr.iter_mut() {
            *item = T::safe_arbitrary(u)?;
        }
        Ok(arr)
    }
}

/// A wrapper type that generates safe HTTP status codes (100-599).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeHttpStatus(pub u16);

impl<'a> SafeArbitrary<'a> for SafeHttpStatus {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(SafeHttpStatus(u.int_in_range(100..=599)?))
    }
}

/// A wrapper type that generates safe email addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeEmail(pub String);

impl<'a> SafeArbitrary<'a> for SafeEmail {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let local_len = u.int_in_range(1..=32)?;
        let domain_len = u.int_in_range(3..=32)?;
        let tld_len = u.int_in_range(2..=6)?;

        let mut local = String::with_capacity(local_len);
        for _ in 0..local_len {
            local.push(u.choose(b"abcdefghijklmnopqrstuvwxyz0123456789._-")? as char);
        }

        let mut domain = String::with_capacity(domain_len);
        for _ in 0..domain_len {
            domain.push(u.choose(b"abcdefghijklmnopqrstuvwxyz0123456789-")? as char);
        }

        let mut tld = String::with_capacity(tld_len);
        for _ in 0..tld_len {
            tld.push(u.choose(b"abcdefghijklmnopqrstuvwxyz")? as char);
        }

        Ok(SafeEmail(format!("{}@{}.{}", local, domain, tld)))
    }
}

/// A wrapper type that generates safe UUID v4 strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeUuid(pub String);

impl<'a> SafeArbitrary<'a> for SafeUuid {
    fn safe_arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut bytes = [0u8; 16];
        u.fill_buffer(&mut bytes)?;
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        let uuid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15]
        );
        Ok(SafeUuid(uuid))
    }
}
