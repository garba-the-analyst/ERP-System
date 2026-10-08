//! FR-4.1 exact decimal money + FR-4.3 HMAC-SHA512 webhook verification.
use hmac::{Hmac, Mac};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha512;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money(pub Decimal);

impl Money {
    pub fn ngn(naira: i64, kobo: u8) -> Self {
        Self(Decimal::new(naira * 100 + kobo as i64, 2))
    }
    /// Spillover fee = per-credit rate × carryover units.
    pub fn spillover_fee(per_credit: Decimal, units: u32) -> Self {
        Self(per_credit * Decimal::from(units))
    }
}

/// Constant-time HMAC-SHA512 check for Paystack/Remita webhooks.
/// `signature` is the hex digest header sent by the gateway.
pub fn verify_hmac_sha512(secret: &[u8], body: &[u8], signature_hex: &str) -> bool {
    let mut mac = match Hmac::<Sha512>::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let expected = mac.finalize().into_bytes();
    let sig = match hex_decode(signature_hex) {
        Some(b) => b,
        None => return false,
    };
    if sig.len() != expected.len() {
        return false;
    }
    // constant-time compare
    let mut diff = 0u8;
    for (a, b) in sig.iter().zip(expected.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    #[test]
    fn no_float_drift() {
        // 0.1 + 0.2 must be exactly 0.30
        let a = Decimal::new(10, 2);
        let b = Decimal::new(20, 2);
        assert_eq!(a + b, dec!(0.30));
    }
    #[test]
    fn hmac_roundtrip() {
        use hmac::Mac;
        let mut mac = Hmac::<Sha512>::new_from_slice(b"key").unwrap();
        mac.update(b"body");
        let hex: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        assert!(verify_hmac_sha512(b"key", b"body", &hex));
        assert!(!verify_hmac_sha512(b"key", b"tampered", &hex));
    }
}
