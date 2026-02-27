//! HOTP implementation per RFC 4226

use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use zeroize::Zeroize;

/// Supported HMAC algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    SHA1,
    SHA256,
    SHA512,
}

/// Generate an HOTP code per RFC 4226
///
/// # Arguments
/// * `secret` - The shared secret key
/// * `counter` - The counter value
/// * `digits` - Number of digits in the output code (6 or 8)
/// * `algorithm` - The HMAC algorithm to use
pub fn generate(secret: &[u8], counter: u64, digits: u32, algorithm: Algorithm) -> String {
    let counter_bytes = counter.to_be_bytes();

    let mut hmac_result = match algorithm {
        Algorithm::SHA1 => {
            let mut mac =
                Hmac::<Sha1>::new_from_slice(secret).expect("HMAC accepts any key length");
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::SHA256 => {
            let mut mac =
                Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts any key length");
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::SHA512 => {
            let mut mac =
                Hmac::<Sha512>::new_from_slice(secret).expect("HMAC accepts any key length");
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
    };

    // Dynamic truncation (RFC 4226 Section 5.4)
    let offset = (hmac_result[hmac_result.len() - 1] & 0x0f) as usize;
    let binary = ((hmac_result[offset] as u32 & 0x7f) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32);

    let otp = binary % 10u32.pow(digits);

    // Zeroize sensitive data
    hmac_result.zeroize();

    format!("{:0>width$}", otp, width = digits as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4226 Appendix D test vectors
    /// Secret: "12345678901234567890" (ASCII)
    #[test]
    fn test_rfc4226_vectors() {
        let secret = b"12345678901234567890";
        let expected = [
            "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583",
            "399871", "520489",
        ];

        for (counter, expected_code) in expected.iter().enumerate() {
            let code = generate(secret, counter as u64, 6, Algorithm::SHA1);
            assert_eq!(
                &code, expected_code,
                "HOTP mismatch at counter={}: expected {}, got {}",
                counter, expected_code, code
            );
        }
    }

    #[test]
    fn test_8_digit_codes() {
        let secret = b"12345678901234567890";
        let code = generate(secret, 0, 8, Algorithm::SHA1);
        assert_eq!(code.len(), 8);
    }

    #[test]
    fn test_different_algorithms() {
        let secret = b"12345678901234567890";
        let sha1 = generate(secret, 0, 6, Algorithm::SHA1);
        let sha256 = generate(secret, 0, 6, Algorithm::SHA256);
        let sha512 = generate(secret, 0, 6, Algorithm::SHA512);

        // Different algorithms should produce different codes
        assert_ne!(sha1, sha256);
        assert_ne!(sha1, sha512);
    }

    #[test]
    fn test_deterministic() {
        let secret = b"test-secret-key";
        let code1 = generate(secret, 42, 6, Algorithm::SHA1);
        let code2 = generate(secret, 42, 6, Algorithm::SHA1);
        assert_eq!(code1, code2);
    }

    #[test]
    fn test_different_counters_produce_different_codes() {
        let secret = b"12345678901234567890";
        let code0 = generate(secret, 0, 6, Algorithm::SHA1);
        let code1 = generate(secret, 1, 6, Algorithm::SHA1);
        assert_ne!(code0, code1);
    }
}
