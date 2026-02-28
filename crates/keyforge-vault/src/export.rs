//! Token export

use crate::constants::OTPAUTH_SCHEME;
use crate::db::Vault;
use crate::error::VaultError;
use zeroize::Zeroize;

impl Vault {
    /// Export all tokens as `otpauth://` URIs (plaintext).
    pub fn export_uris(&self) -> Result<Vec<String>, String> {
        let tokens = self.list_tokens()?;
        let mut uris = Vec::new();

        for token in &tokens {
            let secret = self.get_token_secret(&token.id)?;
            let secret_b32 = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret);

            let mut query_params = Vec::new();
            query_params.push(format!("secret={}", secret_b32));
            query_params.push(format!("algorithm={}", token.algorithm));
            query_params.push(format!("digits={}", token.digits));
            query_params.push(format!("issuer={}", urlencoding_encode(&token.issuer)));

            if token.token_type.eq_ignore_ascii_case("totp") {
                query_params.push(format!("period={}", token.period));
            } else if token.token_type.eq_ignore_ascii_case("hotp") {
                query_params.push(format!("counter={}", token.counter));
            } else {
                // Defensive: for any future/non-standard type, preserve all fields
                query_params.push(format!("period={}", token.period));
                query_params.push(format!("counter={}", token.counter));
            }

            let query = query_params.join("&");

            let uri = format!(
                "{}{}/{}:{}?{}",
                OTPAUTH_SCHEME,
                token.token_type,
                urlencoding_encode(&token.issuer),
                urlencoding_encode(&token.account),
                query,
            );
            uris.push(uri);
        }

        Ok(uris)
    }

    /// Export all tokens as an encrypted JSON blob.
    pub fn export_encrypted(&self, export_password: &[u8]) -> Result<Vec<u8>, String> {
        let uris = self.export_uris()?;
        let json =
            serde_json::to_vec(&uris).map_err(|e| VaultError::Serialization(e.to_string()))?;

        let salt = keyforge_crypto::random::generate_salt();
        let params = keyforge_crypto::kdf::KdfParams::default();
        let mut key = keyforge_crypto::kdf::derive_key(export_password, &salt, &params)?;
        let result = keyforge_crypto::aead::encrypt(&json, &key);
        key.zeroize();
        let encrypted = result?;

        // [salt][encrypted]
        let mut output = Vec::new();
        output.extend_from_slice(&salt);
        output.extend_from_slice(&encrypted);

        Ok(output)
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            ':' => result.push_str("%3A"),
            '@' => result.push_str("%40"),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
