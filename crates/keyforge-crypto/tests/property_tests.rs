use keyforge_crypto::hotp::Algorithm;
use keyforge_crypto::{aead, hotp, totp};
use proptest::prelude::*;

proptest! {
    #[test]
    fn totp_always_returns_correct_length(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        time in 0u64..20000000000,
        digits in prop::sample::select(vec![6u32, 8]),
    ) {
        let code = totp::generate(&secret, time, 30, digits, Algorithm::SHA1);
        prop_assert_eq!(code.len(), digits as usize);
        prop_assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn totp_is_deterministic(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        time in 0u64..20000000000,
    ) {
        let code1 = totp::generate(&secret, time, 30, 6, Algorithm::SHA1);
        let code2 = totp::generate(&secret, time, 30, 6, Algorithm::SHA1);
        prop_assert_eq!(code1, code2);
    }

    #[test]
    fn hotp_always_returns_correct_length(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        counter in 0u64..1000000,
        digits in prop::sample::select(vec![6u32, 8]),
    ) {
        let code = hotp::generate(&secret, counter, digits, Algorithm::SHA1);
        prop_assert_eq!(code.len(), digits as usize);
        prop_assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn aead_encrypt_decrypt_roundtrip(
        plaintext in prop::collection::vec(any::<u8>(), 0..1024),
        key in prop::array::uniform32(any::<u8>()),
    ) {
        let encrypted = aead::encrypt(&plaintext, &key).unwrap();
        let decrypted = aead::decrypt(&encrypted, &key).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aead_wrong_key_fails(
        plaintext in prop::collection::vec(any::<u8>(), 1..256),
        key1 in prop::array::uniform32(any::<u8>()),
        key2 in prop::array::uniform32(any::<u8>()),
    ) {
        prop_assume!(key1 != key2);
        let encrypted = aead::encrypt(&plaintext, &key1).unwrap();
        let result = aead::decrypt(&encrypted, &key2);
        prop_assert!(result.is_err());
    }
}
