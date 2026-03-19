use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(secret: &str, payload: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| anyhow!("invalid HMAC secret"))?;
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn verify_payload(secret: &str, payload: &str, signature: &str) -> Result<bool> {
    let expected = sign_payload(secret, payload)?;
    Ok(constant_time_eq(expected.as_bytes(), signature.as_bytes()))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{sign_payload, verify_payload};

    #[test]
    fn signs_and_verifies_payload() -> Result<()> {
        let signature = sign_payload("secret", "payload")?;
        assert!(verify_payload("secret", "payload", &signature)?);
        assert!(!verify_payload("secret", "payload", "bad")?);
        Ok(())
    }
}
