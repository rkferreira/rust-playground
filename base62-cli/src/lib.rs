//! Base62 encoding/decoding utilities without external base62 crate.

use anyhow::Result;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{Zero, ToPrimitive};

/// URL‑safe Base62 alphabet.
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encode arbitrary binary data into a Base62 string.
/// Returns an empty string for empty input.
pub fn encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    // Preserve leading zero bytes as '0' characters in the output.
    let leading_zeros = data.iter().take_while(|&&b| b == 0).count();
    let non_zero = &data[leading_zeros..];
    if non_zero.is_empty() {
        return "0".repeat(leading_zeros);
    }
    let mut num = BigUint::from_bytes_be(non_zero);
    let base = BigUint::from(62u32);
    let mut chars = Vec::new();
    while !num.is_zero() {
        let (q, r) = num.div_mod_floor(&base);
        let idx = r.to_usize().unwrap();
        chars.push(ALPHABET[idx] as char);
        num = q;
    }
    let encoded = chars.iter().rev().collect::<String>();
    "0".repeat(leading_zeros) + &encoded
}


/// Decode a Base62 string back into raw bytes.
/// Returns an `anyhow::Error` if the string contains invalid characters.
pub fn decode(s: &str) -> Result<Vec<u8>> {
    // Count leading '0' characters which represent zero bytes.
    let leading_zeros = s.chars().take_while(|c| *c == '0').count();
    let non_zero_part = &s[leading_zeros..];
    if non_zero_part.is_empty() {
        return Ok(vec![0u8; leading_zeros]);
    }
    let base = BigUint::from(62u32);
    let mut num = BigUint::zero();
    for c in non_zero_part.bytes() {
        let val = ALPHABET.iter().position(|&b| b == c)
            .ok_or_else(|| anyhow::anyhow!("Invalid Base62 character: {}", c as char))?;
        num = num * &base + BigUint::from(val);
    }
    let mut decoded = num.to_bytes_be();
    if leading_zeros > 0 {
        let mut with_zeros = vec![0u8; leading_zeros];
        with_zeros.extend_from_slice(&decoded);
        decoded = with_zeros;
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let enc = encode(data);
        let dec = decode(&enc).expect("decode should succeed");
        assert_eq!(dec, data);
    }

    #[test]
    fn test_encode_empty() {
        let data: &[u8] = &[];
        let enc = encode(data);
        assert_eq!(enc, "");
        let dec = decode(&enc).expect("decode empty should succeed");
        assert_eq!(dec, data);
    }

    #[test]
    fn test_decode_invalid() {
        let invalid = "!@#";
        let err = decode(invalid).unwrap_err();
        assert!(err.to_string().contains("Invalid Base62 character"));
    }

    #[test]
    fn test_large_input() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let enc = encode(&data);
        let dec = decode(&enc).expect("decode large input");
        assert_eq!(dec, data);
    }
}
