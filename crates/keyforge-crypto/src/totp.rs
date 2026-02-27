//! TOTP implementation per RFC 6238

use crate::hotp;

pub use hotp::Algorithm;

/// Generate a TOTP code per RFC 6238.
pub fn generate(
    secret: &[u8],
    time: u64,
    period: u64,
    digits: u32,
    algorithm: Algorithm,
) -> String {
    let counter = time / period;
    hotp::generate(secret, counter, digits, algorithm)
}

/// Seconds remaining in the current TOTP period.
pub fn time_remaining(time: u64, period: u64) -> u64 {
    period - (time % period)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 test vectors
    // Secret for SHA1: "12345678901234567890" (20 bytes)
    // Secret for SHA256: "12345678901234567890123456789012" (32 bytes)
    // Secret for SHA512: "1234567890123456789012345678901234567890123456789012345678901234" (64 bytes)

    fn sha1_secret() -> &'static [u8] {
        b"12345678901234567890"
    }

    fn sha256_secret() -> &'static [u8] {
        b"12345678901234567890123456789012"
    }

    fn sha512_secret() -> &'static [u8] {
        b"1234567890123456789012345678901234567890123456789012345678901234"
    }

    #[test]
    fn test_rfc6238_vectors_8_digits() {
        let test_cases = [
            (59u64, Algorithm::SHA1, "94287082", sha1_secret() as &[u8]),
            (59, Algorithm::SHA256, "46119246", sha256_secret()),
            (59, Algorithm::SHA512, "90693936", sha512_secret()),
            (1111111109, Algorithm::SHA1, "07081804", sha1_secret()),
            (1111111109, Algorithm::SHA256, "68084774", sha256_secret()),
            (1111111109, Algorithm::SHA512, "25091201", sha512_secret()),
            (1111111111, Algorithm::SHA1, "14050471", sha1_secret()),
            (1111111111, Algorithm::SHA256, "67062674", sha256_secret()),
            (1111111111, Algorithm::SHA512, "99943326", sha512_secret()),
            (1234567890, Algorithm::SHA1, "89005924", sha1_secret()),
            (1234567890, Algorithm::SHA256, "91819424", sha256_secret()),
            (1234567890, Algorithm::SHA512, "93441116", sha512_secret()),
            (2000000000, Algorithm::SHA1, "69279037", sha1_secret()),
            (2000000000, Algorithm::SHA256, "90698825", sha256_secret()),
            (2000000000, Algorithm::SHA512, "38618901", sha512_secret()),
            (20000000000, Algorithm::SHA1, "65353130", sha1_secret()),
            (20000000000, Algorithm::SHA256, "77737706", sha256_secret()),
            (20000000000, Algorithm::SHA512, "47863826", sha512_secret()),
        ];

        for (time, algorithm, expected, secret) in &test_cases {
            let code = generate(secret, *time, 30, 8, *algorithm);
            assert_eq!(
                &code, expected,
                "TOTP mismatch at time={}, algo={:?}: expected {}, got {}",
                time, algorithm, expected, code
            );
        }
    }

    #[test]
    fn test_6_digit_codes() {
        let secret = sha1_secret();
        let code = generate(secret, 59, 30, 6, Algorithm::SHA1);
        assert_eq!(code.len(), 6);
        // 94287082 truncated to 6 digits = last 6: 287082
        assert_eq!(code, "287082");
    }

    #[test]
    fn test_time_remaining() {
        assert_eq!(time_remaining(0, 30), 30);
        assert_eq!(time_remaining(1, 30), 29);
        assert_eq!(time_remaining(29, 30), 1);
        assert_eq!(time_remaining(30, 30), 30);
        assert_eq!(time_remaining(31, 30), 29);
    }

    #[test]
    fn test_different_periods() {
        let secret = sha1_secret();
        let code_30 = generate(secret, 59, 30, 6, Algorithm::SHA1);
        let code_60 = generate(secret, 59, 60, 6, Algorithm::SHA1);
        // Different periods may produce different codes
        // At time=59, period=30 -> counter=1, period=60 -> counter=0
        assert_ne!(code_30, code_60);
    }

    #[test]
    fn test_deterministic() {
        let secret = sha1_secret();
        let code1 = generate(secret, 1000, 30, 6, Algorithm::SHA1);
        let code2 = generate(secret, 1000, 30, 6, Algorithm::SHA1);
        assert_eq!(code1, code2);
    }

    #[test]
    fn test_same_period_same_code() {
        let secret = sha1_secret();
        // Times within the same period should produce the same code
        let code1 = generate(secret, 30, 30, 6, Algorithm::SHA1);
        let code2 = generate(secret, 31, 30, 6, Algorithm::SHA1);
        let code3 = generate(secret, 59, 30, 6, Algorithm::SHA1);
        assert_eq!(code1, code2);
        assert_eq!(code2, code3);
    }
}
