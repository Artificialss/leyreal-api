//! DEMO of the real auth pattern -- no database, no real key storage.
//!
//! The production API (leyreal-system, private) hashes the raw key with
//! SHA-256 and looks the hash up in `api_keys.key_hash`, checks
//! `is_active`/`revoked_at`, rejects anything not tier `paid`, and
//! enforces per-key daily/monthly quotas against `api_key_usage_daily`.
//! This demo keeps the real hashing step (it's harmless to show) but
//! replaces the lookup with a hardcoded acceptance of any non-empty
//! Bearer token, and fabricates a demo key context instead of a real row.

use sha2::{Digest, Sha256};
use vercel_runtime::Request;

pub struct DemoKeyContext {
    pub raw_key: String,
    pub opaque_id_salt: Vec<u8>,
}

fn hash_key(raw_key: &str) -> String {
    let digest = Sha256::digest(raw_key.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_bearer(req: &Request) -> Option<String> {
    let header = req.headers().get("authorization")?.to_str().ok()?;
    header.strip_prefix("Bearer ").map(|s| s.trim().to_string())
}

pub fn authenticate_demo(req: &Request) -> Result<DemoKeyContext, String> {
    let raw_key = extract_bearer(req)
        .ok_or_else(|| "Missing Authorization: Bearer <api-key> header.".to_string())?;

    // Real code: `SELECT ... FROM api_keys WHERE key_hash = $1`.
    let _key_hash_would_be = hash_key(&raw_key);

    Ok(DemoKeyContext {
        raw_key,
        // Real code: a random 16-byte salt generated once per key at
        // creation (`pgcrypto`'s gen_random_bytes) and stored on the row.
        // This demo uses a fixed constant since there's no row to store
        // one on.
        opaque_id_salt: b"demo-salt-000000".to_vec(),
    })
}
