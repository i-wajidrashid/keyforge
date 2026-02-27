//! Secure random number generation

use rand::RngCore;

/// Generate cryptographically secure random bytes
pub fn generate_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

/// Generate a random 16-byte salt
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a random 12-byte nonce for AES-256-GCM
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bytes_length() {
        for len in [0, 1, 16, 32, 64, 128, 256] {
            let bytes = generate_bytes(len);
            assert_eq!(bytes.len(), len);
        }
    }

    #[test]
    fn test_generate_salt_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn test_generate_nonce_length() {
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_randomness() {
        // Two calls should not produce identical output (probabilistic check)
        let mut all_same = true;
        for _ in 0..100 {
            let a = generate_bytes(32);
            let b = generate_bytes(32);
            if a != b {
                all_same = false;
                break;
            }
        }
        assert!(
            !all_same,
            "100 consecutive random generations should not all be identical"
        );
    }

    #[test]
    fn test_salt_randomness() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_ne!(salt1, salt2);
    }
}
