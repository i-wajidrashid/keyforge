//! Import tokens from various sources

use crate::db::Vault;
use crate::token::NewToken;

impl Vault {
    /// Import tokens from a list of otpauth:// URIs
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

    /// Import from encrypted KeyForge export
    pub fn import_encrypted(&self, data: &[u8], password: &[u8]) -> Result<usize, String> {
        if data.len() < 16 {
            return Err("Invalid export file".to_string());
        }
        let (salt_bytes, encrypted) = data.split_at(16);
        let mut salt = [0u8; 16];
        salt.copy_from_slice(salt_bytes);

        let params = keyforge_crypto::kdf::KdfParams::default();
        let key = keyforge_crypto::kdf::derive_key(password, &salt, &params)?;
        let json = keyforge_crypto::aead::decrypt(encrypted, &key)?;

        let uris: Vec<String> = serde_json::from_slice(&json)
            .map_err(|e| format!("Failed to parse export: {}", e))?;

        self.import_uris(&uris)
    }
}

/// Parse an otpauth:// URI into a NewToken
pub fn parse_otpauth_uri(uri: &str) -> Result<Option<NewToken>, String> {
    if !uri.starts_with("otpauth://") {
        return Err(format!("Invalid otpauth URI: {}", uri));
    }

    let without_scheme = &uri[10..]; // Remove "otpauth://"
    let (token_type, rest) = without_scheme.split_once('/')
        .ok_or_else(|| "Missing token type in URI".to_string())?;

    let token_type = match token_type {
        "totp" => "totp".to_string(),
        "hotp" => "hotp".to_string(),
        _ => return Err(format!("Unknown token type: {}", token_type)),
    };

    let (label, query) = rest.split_once('?')
        .ok_or_else(|| "Missing query parameters in URI".to_string())?;

    // Parse label (issuer:account or just account)
    let label = urlencoding_decode(label);
    let (issuer_from_label, account) = if let Some((issuer, account)) = label.split_once(':') {
        (Some(issuer.to_string()), account.to_string())
    } else {
        (None, label.to_string())
    };

    // Parse query parameters
    let params: std::collections::HashMap<String, String> = query
        .split('&')
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            Some((k.to_lowercase(), urlencoding_decode(v)))
        })
        .collect();

    let secret_b32 = params.get("secret")
        .ok_or_else(|| "Missing secret parameter".to_string())?;

    let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &secret_b32.to_uppercase())
        .ok_or_else(|| "Invalid base32 secret".to_string())?;

    let issuer = params.get("issuer")
        .map(|s| s.to_string())
        .or(issuer_from_label)
        .unwrap_or_else(|| "Unknown".to_string());

    let algorithm = params.get("algorithm")
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "SHA1".to_string());

    let digits: u32 = params.get("digits")
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);

    let period: u32 = params.get("period")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let counter: u64 = params.get("counter")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

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
    let mut bytes = Vec::new();
    let mut chars = s.as_bytes().iter();
    while let Some(&b) = chars.next() {
        if b == b'%' {
            let hex: Vec<u8> = chars.by_ref().take(2).copied().collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(
                    &String::from_utf8_lossy(&hex),
                    16,
                ) {
                    bytes.push(byte);
                }
            }
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
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
