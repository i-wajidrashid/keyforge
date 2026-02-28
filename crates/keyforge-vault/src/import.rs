//! Token import

use crate::constants::{
    DEFAULT_ALGORITHM, DEFAULT_COUNTER, DEFAULT_DIGITS, DEFAULT_ISSUER, DEFAULT_PERIOD,
    EXPORT_SALT_SIZE, OTPAUTH_SCHEME, OTPAUTH_SCHEME_LEN, TOKEN_TYPE_HOTP, TOKEN_TYPE_TOTP,
};
use crate::db::Vault;
use crate::error::VaultError;
use crate::token::NewToken;
use zeroize::Zeroize;

impl Vault {
    /// Import tokens from `otpauth://` URIs.
    pub fn import_uris(&self, uris: &[String]) -> Result<usize, String> {
        let mut count = 0;
        for uri in uris {
            if let Some(token) = parse_otpauth_uri(uri)? {
                self.add_token(token)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import from an encrypted KeyForge export.
    pub fn import_encrypted(&self, data: &[u8], password: &[u8]) -> Result<usize, String> {
        if data.len() < EXPORT_SALT_SIZE {
            return Err(VaultError::InvalidExportFile.to_string());
        }
        let (salt_bytes, encrypted) = data.split_at(EXPORT_SALT_SIZE);
        let mut salt = [0u8; EXPORT_SALT_SIZE];
        salt.copy_from_slice(salt_bytes);

        let params = keyforge_crypto::kdf::KdfParams::default();
        let mut key = keyforge_crypto::kdf::derive_key(password, &salt, &params)?;
        let result = keyforge_crypto::aead::decrypt(encrypted, &key);
        key.zeroize();
        let json = result?;

        let uris: Vec<String> =
            serde_json::from_slice(&json).map_err(|e| VaultError::Serialization(e.to_string()))?;

        self.import_uris(&uris)
    }
}

/// Parse an `otpauth://` URI into a NewToken.
pub fn parse_otpauth_uri(uri: &str) -> Result<Option<NewToken>, String> {
    if !uri.starts_with(OTPAUTH_SCHEME) {
        return Err(VaultError::InvalidUri(uri.to_string()).to_string());
    }

    let without_scheme = &uri[OTPAUTH_SCHEME_LEN..];
    let (token_type, rest) = without_scheme
        .split_once('/')
        .ok_or_else(|| VaultError::InvalidUri("missing token type".to_string()))?;

    let token_type = match token_type {
        t if t == TOKEN_TYPE_TOTP => TOKEN_TYPE_TOTP.to_string(),
        t if t == TOKEN_TYPE_HOTP => TOKEN_TYPE_HOTP.to_string(),
        _ => return Err(VaultError::UnknownTokenType(token_type.to_string()).to_string()),
    };

    let (label, query) = rest
        .split_once('?')
        .ok_or_else(|| VaultError::InvalidUri("missing query parameters".to_string()))?;

    // label = "issuer:account" or just "account"
    let label = urlencoding_decode(label);
    let (issuer_from_label, account) = if let Some((issuer, account)) = label.split_once(':') {
        (Some(issuer.to_string()), account.to_string())
    } else {
        (None, label.to_string())
    };

    let params: std::collections::HashMap<String, String> = query
        .split('&')
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.to_lowercase(), urlencoding_decode(v)))
        })
        .collect();

    let secret_b32 = params
        .get("secret")
        .ok_or(VaultError::MissingUriParam("secret"))?;

    let secret = base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &secret_b32.to_uppercase(),
    )
    .ok_or(VaultError::InvalidBase32Secret)?;

    let issuer = params
        .get("issuer")
        .map(|s| s.to_string())
        .or(issuer_from_label)
        .unwrap_or_else(|| DEFAULT_ISSUER.to_string());

    let algorithm = params
        .get("algorithm")
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| DEFAULT_ALGORITHM.to_string());

    // Validate algorithm
    match algorithm.as_str() {
        "SHA1" | "SHA256" | "SHA512" => {}
        other => {
            return Err(VaultError::InvalidUri(format!("unsupported algorithm: {other}")).into())
        }
    }

    let digits: u32 = params
        .get("digits")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_DIGITS);

    // Validate digits (only 6 or 8 per RFC 4226 / HOTP spec)
    if digits != 6 && digits != 8 {
        return Err(VaultError::InvalidUri(format!("unsupported digits: {digits}")).into());
    }

    let period: u32 = params
        .get("period")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PERIOD);

    // Validate period is positive
    if period == 0 {
        return Err(VaultError::InvalidUri("period must be > 0".to_string()).into());
    }

    let counter: u64 = params
        .get("counter")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_COUNTER);

    Ok(Some(NewToken {
        issuer,
        account,
        secret,
        algorithm,
        digits,
        token_type,
        period,
        counter,
        icon: None,
    }))
}

fn urlencoding_decode(s: &str) -> String {
    let bytes_in = s.as_bytes();
    let mut bytes_out = Vec::with_capacity(bytes_in.len());
    let mut i = 0;

    while i < bytes_in.len() {
        let b = bytes_in[i];
        if b == b'%' {
            if i + 2 < bytes_in.len() {
                let h1 = bytes_in[i + 1] as char;
                let h2 = bytes_in[i + 2] as char;
                if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() {
                    let hex_str = &s[i + 1..=i + 2];
                    if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                        bytes_out.push(byte);
                        i += 3;
                        continue;
                    }
                }
            }
            // Invalid or incomplete percent-escape: keep '%' as-is
            bytes_out.push(b'%');
            i += 1;
        } else if b == b'+' {
            bytes_out.push(b' ');
            i += 1;
        } else {
            bytes_out.push(b);
            i += 1;
        }
    }

    String::from_utf8(bytes_out)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_totp_uri() {
        let uri = "otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&algorithm=SHA1&digits=6&period=30";
        let token = parse_otpauth_uri(uri).unwrap().unwrap();
        assert_eq!(token.issuer, "GitHub");
        assert_eq!(token.account, "user@example.com");
        assert_eq!(token.algorithm, "SHA1");
        assert_eq!(token.digits, 6);
        assert_eq!(token.period, 30);
        assert_eq!(token.token_type, "totp");
    }

    #[test]
    fn test_parse_hotp_uri() {
        let uri = "otpauth://hotp/Test:user?secret=JBSWY3DPEHPK3PXP&counter=42";
        let token = parse_otpauth_uri(uri).unwrap().unwrap();
        assert_eq!(token.token_type, "hotp");
        assert_eq!(token.counter, 42);
    }

    #[test]
    fn test_parse_defaults() {
        let uri = "otpauth://totp/user?secret=JBSWY3DPEHPK3PXP";
        let token = parse_otpauth_uri(uri).unwrap().unwrap();
        assert_eq!(token.issuer, "Unknown");
        assert_eq!(token.algorithm, "SHA1");
        assert_eq!(token.digits, 6);
        assert_eq!(token.period, 30);
    }

    #[test]
    fn test_parse_invalid_uri() {
        let result = parse_otpauth_uri("https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_secret() {
        let result = parse_otpauth_uri("otpauth://totp/Test?algorithm=SHA1");
        assert!(result.is_err());
    }
}
