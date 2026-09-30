//! Per-key, reversible, stateless obfuscation of a bigint database id --
//! see leyreal-costa-rica/db/opaque_ids.md for the full design rationale.
//! No id-mapping table: given the same raw API key and the stored
//! opaque_id_salt, the transform runs backwards to recover the real id.

use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const HKDF_INFO: &[u8] = b"leyreal-opaque-id-v1";
const ROUNDS: u8 = 8;
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn derive_cipher_key(raw_api_key: &str, opaque_id_salt: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(opaque_id_salt), raw_api_key.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(HKDF_INFO, &mut okm)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    okm
}

fn round_function(cipher_key: &[u8; 32], round: u8, half: u32) -> u32 {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(cipher_key).expect("HMAC-SHA256 accepts any key length");
    let mut input = [0u8; 5];
    input[0] = round;
    input[1..5].copy_from_slice(&half.to_be_bytes());
    mac.update(&input);
    let digest = mac.finalize().into_bytes();
    u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]])
}

fn feistel_encode(cipher_key: &[u8; 32], value: u64) -> u64 {
    let mut left = (value >> 32) as u32;
    let mut right = value as u32;
    for round in 0..ROUNDS {
        let new_left = right;
        let new_right = left ^ round_function(cipher_key, round, right);
        left = new_left;
        right = new_right;
    }
    ((left as u64) << 32) | (right as u64)
}

fn feistel_decode(cipher_key: &[u8; 32], value: u64) -> u64 {
    let mut left = (value >> 32) as u32;
    let mut right = value as u32;
    for round in (0..ROUNDS).rev() {
        let new_right = left;
        let new_left = right ^ round_function(cipher_key, round, left);
        left = new_left;
        right = new_right;
    }
    ((left as u64) << 32) | (right as u64)
}

fn to_base62(mut value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while value > 0 {
        out.push(BASE62_ALPHABET[(value % 62) as usize]);
        value /= 62;
    }
    out.reverse();
    String::from_utf8(out).expect("base62 alphabet is ASCII")
}

fn from_base62(text: &str) -> Result<u64, String> {
    let mut v: u64 = 0;
    for ch in text.bytes() {
        let idx = BASE62_ALPHABET
            .iter()
            .position(|&c| c == ch)
            .ok_or_else(|| format!("Invalid opaque id character: {}", ch as char))?;
        v = v
            .checked_mul(62)
            .and_then(|v| v.checked_add(idx as u64))
            .ok_or_else(|| "opaque id overflow".to_string())?;
    }
    Ok(v)
}

pub fn encode(raw_api_key: &str, opaque_id_salt: &[u8], id: i64) -> String {
    let cipher_key = derive_cipher_key(raw_api_key, opaque_id_salt);
    to_base62(feistel_encode(&cipher_key, id as u64))
}

pub fn decode(raw_api_key: &str, opaque_id_salt: &[u8], opaque_id: &str) -> Result<i64, String> {
    let cipher_key = derive_cipher_key(raw_api_key, opaque_id_salt);
    let encoded = from_base62(opaque_id)?;
    Ok(feistel_decode(&cipher_key, encoded) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let salt = b"0123456789abcdef";
        let key = "sk_test_abcdef";
        for id in [0i64, 1, 42, 12345, 99858, i64::from(u32::MAX)] {
            let encoded = encode(key, salt, id);
            assert_eq!(decode(key, salt, &encoded).unwrap(), id);
        }
    }

    #[test]
    fn different_keys_produce_different_tokens() {
        let salt = b"0123456789abcdef";
        assert_ne!(encode("key-a", salt, 12345), encode("key-b", salt, 12345));
    }
}
