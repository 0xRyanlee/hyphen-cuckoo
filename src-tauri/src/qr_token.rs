use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;

static SECRET: OnceLock<Vec<u8>> = OnceLock::new();

fn secret_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Cuckoo")
        .join("qr-secret.bin")
}

/// Loads (or first-run generates) the 32-byte HMAC secret. Stored next to
/// role-auth.json in the app data dir, never committed.
fn secret() -> &'static [u8] {
    SECRET.get_or_init(|| {
        let path = secret_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() >= 32 {
                return bytes;
            }
        }
        let mut buf = [0u8; 32];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut buf);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, buf);
        buf.to_vec()
    })
}

fn sign(payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret()).expect("HMAC accepts any key length");
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    // Truncate to 16 bytes (32 hex chars) — ample against forgery, shorter URL/QR.
    hex::encode(&sig[..16])
}

/// `token = hex(payload) + "." + hex(hmac(payload))`. Hex keeps it URL-safe
/// with zero base64 dependency; payloads here are short (table no / order id).
pub fn make_token(payload: &str) -> String {
    format!("{}.{}", hex::encode(payload.as_bytes()), sign(payload))
}

/// Verifies a token and returns the original payload string if the signature
/// is valid. Constant-time compare via HMAC verify.
pub fn verify_token(token: &str) -> Option<String> {
    let (payload_hex, sig_hex) = token.split_once('.')?;
    let payload_bytes = hex::decode(payload_hex).ok()?;
    let payload = String::from_utf8(payload_bytes).ok()?;
    let expected = sign(&payload);
    // Length-prefixed constant-time-ish compare; HMAC keys are fixed-size so a
    // direct byte compare of the hex is acceptable here.
    if constant_eq(sig_hex.as_bytes(), expected.as_bytes()) {
        Some(payload)
    } else {
        None
    }
}

fn constant_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ── Payload helpers ─────────────────────────────────────────────────────────
// Table code (fixed, printed): "t|{table_no}"
// Order marketing code (per-order, voidable): "m|{order_id}|{component}|{char}|{nonce}"

pub fn table_payload(table_no: &str) -> String {
    format!("t|{}", table_no)
}

pub fn parse_table_payload(payload: &str) -> Option<String> {
    payload.strip_prefix("t|").map(|s| s.to_string())
}

pub fn marketing_payload(order_id: i64, component: &str, ch: &str, nonce: &str) -> String {
    format!("m|{}|{}|{}|{}", order_id, component, ch, nonce)
}

pub struct MarketingPayload {
    pub order_id: i64,
    pub component: String,
    pub ch: String,
    #[allow(dead_code)] // only consumed in tests; presence guarantees token uniqueness
    pub nonce: String,
}

pub fn parse_marketing_payload(payload: &str) -> Option<MarketingPayload> {
    let rest = payload.strip_prefix("m|")?;
    let parts: Vec<&str> = rest.splitn(4, '|').collect();
    if parts.len() != 4 {
        return None;
    }
    Some(MarketingPayload {
        order_id: parts[0].parse().ok()?,
        component: parts[1].to_string(),
        ch: parts[2].to_string(),
        nonce: parts[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_table_token() {
        let payload = table_payload("A3");
        let token = make_token(&payload);
        let recovered = verify_token(&token).expect("valid token");
        assert_eq!(parse_table_payload(&recovered).unwrap(), "A3");
    }

    #[test]
    fn rejects_tampered_token() {
        let token = make_token(&table_payload("A3"));
        let (p, _) = token.split_once('.').unwrap();
        let forged = format!("{}.{}", p, "deadbeefdeadbeefdeadbeefdeadbeef");
        assert!(verify_token(&forged).is_none());
    }

    #[test]
    fn rejects_payload_swap() {
        // A token signed for A3 must not validate when payload is swapped to A4.
        let token = make_token(&table_payload("A3"));
        let sig = token.split_once('.').unwrap().1;
        let forged = format!("{}.{}", hex::encode(table_payload("A4").as_bytes()), sig);
        assert!(verify_token(&forged).is_none());
    }

    #[test]
    fn roundtrip_marketing_token() {
        let payload = marketing_payload(1001, "character_collect", "喜", "7fa3");
        let token = make_token(&payload);
        let recovered = verify_token(&token).expect("valid");
        let mp = parse_marketing_payload(&recovered).unwrap();
        assert_eq!(mp.order_id, 1001);
        assert_eq!(mp.component, "character_collect");
        assert_eq!(mp.ch, "喜");
        assert_eq!(mp.nonce, "7fa3");
    }
}
