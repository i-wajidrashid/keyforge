# Security

Everything about how KeyForge protects secrets. Read this document FULLY before writing any code that touches cryptography, vault storage, or secret material.

---

## Threat Model

### What We Protect

1. **TOTP/HOTP secrets** — The shared keys that generate codes. If leaked, an attacker can generate valid 2FA codes indefinitely.
2. **Token metadata** — Issuer names, account names. Leaking these reveals which services the user has 2FA enabled on (information leakage).
3. **Master password** — Never stored. Only exists in memory during vault unlock and key derivation.
4. **Derived vault key** — The AES-256 key used to encrypt/decrypt the vault. Stored in platform keychain for biometric unlock. Zeroized from memory on vault lock.

### Threat Actors

| Threat | Mitigation |
|--------|-----------|
| Device theft (locked) | Vault encrypted at rest with AES-256. Attacker needs master password or biometric. |
| Device theft (unlocked, app in background) | Auto-lock after configurable timeout (default: 60 seconds). Lock on screen off. |
| Malware reading files | SQLCipher encrypts the entire database file. Raw file is indistinguishable from random data without the key. |
| Malware reading memory | Zeroize secret material from memory after use. Minimize time secrets exist in plaintext. Rust's `zeroize` crate for this. |
| Brute-force master password | Argon2id with high memory/time cost makes offline brute-force extremely expensive. |
| Shoulder surfing | Codes auto-hide after copy. Optional "tap to reveal" mode. |
| Clipboard sniffing | Auto-clear clipboard after configurable timeout (default: 30 seconds). |
| Network eavesdropping | Phase 1 makes ZERO network calls. There is no network attack surface. |
| Supply chain attack | Minimal dependencies. Rust crypto from audited crates only. Reproducible builds (goal). |
| Extension store compromise | Extension code is open source, verifiable. Content Security Policy locked down. No remote code execution. |

### What We Explicitly Do NOT Protect Against

- **Root/admin access on the device** — If the attacker has root, they can read process memory, keychain, everything. This is out of scope for any authenticator app.
- **Compromised OS keychain** — We trust the OS keychain. If it is compromised, the entire device security model is broken.
- **User choosing a weak master password** — We can warn (and SHOULD warn) but cannot force password strength.

---

## Vault Encryption

### Overview

The vault is a single SQLCipher database file per device. SQLCipher provides transparent, page-level AES-256-CBC encryption of the entire database file with HMAC-SHA512 page authentication.

### Encryption Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Cipher | AES-256-CBC (SQLCipher default) | Industry standard, hardware-accelerated on most platforms |
| Page authentication | HMAC-SHA512 | Detects tampering of individual database pages |
| Page size | 4096 bytes (SQLCipher default) | Balance between performance and granularity |
| KDF iterations | SQLCipher default (256000 for SQLCipher 4.x) | Sufficient for database key derivation |

### Database Schema (Phase 1)

Table: `tokens`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PRIMARY KEY | UUID v4 |
| issuer | TEXT NOT NULL | Service name (e.g., "GitHub") |
| account | TEXT NOT NULL | Account identifier (e.g., "user@email.com") |
| secret_encrypted | BLOB NOT NULL | Secret key, additionally encrypted with AES-256-GCM (double encryption layer) |
| algorithm | TEXT NOT NULL DEFAULT 'SHA1' | HMAC algorithm: SHA1, SHA256, SHA512 |
| digits | INTEGER NOT NULL DEFAULT 6 | Code length: 6 or 8 |
| type | TEXT NOT NULL DEFAULT 'totp' | Token type: totp or hotp |
| period | INTEGER NOT NULL DEFAULT 30 | TOTP period in seconds |
| counter | INTEGER NOT NULL DEFAULT 0 | HOTP counter (only used for HOTP tokens) |
| icon | TEXT | Optional custom icon identifier |
| sort_order | INTEGER NOT NULL DEFAULT 0 | User-defined display order |
| created_at | TEXT NOT NULL | ISO 8601 timestamp |
| updated_at | TEXT NOT NULL | ISO 8601 timestamp |
| last_modified | TEXT | Sync timestamp (nullable, for Phase 3) |
| device_id | TEXT | Originating device (nullable, for Phase 3) |
| sync_version | INTEGER | CRDT version counter (nullable, for Phase 3) |

Table: `vault_meta`

| Column | Type | Description |
|--------|------|-------------|
| key | TEXT PRIMARY KEY | Metadata key |
| value | TEXT NOT NULL | Metadata value |

Stores: schema_version, vault_created_at, last_locked_at, device_id.

Table: `migrations`

| Column | Type | Description |
|--------|------|-------------|
| version | INTEGER PRIMARY KEY | Migration version number |
| applied_at | TEXT NOT NULL | When the migration was applied |

### Double Encryption Layer

The vault database itself is encrypted via SQLCipher (first layer). Additionally, the `secret_encrypted` column stores secrets that are ALSO encrypted with AES-256-GCM using a separate key derived from the master password (second layer).

Why double encrypt?

1. If SQLCipher has a vulnerability, the secrets still have their own encryption
2. The inner encryption uses AES-256-GCM (authenticated encryption with associated data) which provides integrity verification per-secret
3. Each secret has its own unique nonce, preventing pattern analysis even within the database
4. When exporting, secrets can be re-encrypted with a different key without touching the vault encryption

### Secret Encryption Format (Inner Layer)

Each `secret_encrypted` blob is structured as:

```
[12 bytes nonce][N bytes ciphertext][16 bytes GCM auth tag]
```

- Nonce: 12 bytes, randomly generated per secret, per write
- Ciphertext: AES-256-GCM encrypted secret bytes
- Auth tag: 16 bytes, GCM authentication tag
- Key: derived from master password using Argon2id (separate derivation from the SQLCipher key)

---

## Key Derivation

### Master Password to Vault Key

When the user enters their master password, two keys are derived:

1. **SQLCipher database key** — Used to open the encrypted database
2. **Secret encryption key** — Used to decrypt the inner `secret_encrypted` blobs

Both are derived using Argon2id with DIFFERENT salts so that compromising one does not reveal the other.

### Argon2id Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Algorithm | Argon2id | Hybrid: resistant to both side-channel and GPU attacks |
| Memory | 64 MiB (65536 KiB) | High enough to resist GPU attacks, low enough for mobile devices |
| Time (iterations) | 3 | Balance between security and unlock speed |
| Parallelism | 4 | Matches common device core counts |
| Salt length | 16 bytes | Randomly generated on vault creation, stored in vault file header |
| Output length | 32 bytes | 256-bit key |

These parameters SHOULD be tunable. On first vault creation, the app SHOULD benchmark the device and adjust memory/time parameters to target ~500ms unlock time. Store the chosen parameters alongside the salt.

### Salt Storage

The Argon2id salts (one per derived key) are stored in a small unencrypted header prepended to the vault file, or in a separate `.keyforge-meta` file alongside the vault. The salt is NOT secret — its purpose is to prevent precomputed rainbow table attacks.

```
[4 bytes magic "KFVT"][2 bytes version][16 bytes sqlcipher_salt][16 bytes secret_salt][4 bytes argon2_memory_kib][1 byte argon2_time][1 byte argon2_parallelism]
```

---

## Platform Keychain Integration

For biometric unlock (Face ID, Touch ID, fingerprint), the derived vault key is stored in the platform's secure enclave / keychain so the user doesn't need to re-enter their master password every time.

### Per-Platform Storage

| Platform | Secure Storage | Access Control |
|----------|---------------|----------------|
| macOS | Keychain Services (kSecClassGenericPassword) | Require user presence (biometric or device password) |
| Windows | Windows Credential Manager (or Windows Hello) | Require Windows Hello authentication |
| Linux | Secret Service API (libsecret) via GNOME Keyring or KWallet | Session-level access |
| Android | Android Keystore (hardware-backed on supported devices) | BiometricPrompt required |
| iOS | iOS Keychain (Secure Enclave on devices with it) | LAContext biometric policy |
| Chrome Extension | NOT AVAILABLE | Master password only, every time |

### Chrome Extension: No Keychain

The Chrome extension CANNOT access platform keychains. The only option is:

1. User enters master password every time the extension popup opens
2. OR session-based unlock: derive the key, hold it in `chrome.storage.session` (in-memory, cleared on browser close), re-prompt after configurable timeout

`chrome.storage.session` is in-memory only (max 10 MB quota), not written to disk. This is the best available option for extensions. The key MUST be cleared when:

- The browser is closed
- The extension's configurable auto-lock timeout expires
- The user manually locks

---

## Memory Safety

### Zeroization

All secret material in Rust MUST be zeroized from memory when no longer needed:

- Use the `zeroize` crate's `Zeroize` and `ZeroizeOnDrop` traits
- Master password buffer: zeroized immediately after key derivation
- Derived keys: zeroized on vault lock
- Decrypted secrets: zeroized after TOTP/HOTP code generation
- Temporary buffers in crypto operations: zeroized after use

### Frontend Memory

TypeScript/JavaScript does not support reliable memory zeroization (strings are immutable, garbage collection is non-deterministic). Therefore:

- Secrets SHOULD only exist in Rust memory on Tauri platforms
- The frontend SHOULD receive only the generated CODE (6-8 digits), never the raw secret
- On the Chrome extension (where Rust is unavailable), secrets exist in JS memory — this is an accepted tradeoff. Minimize the time they're held. Clear references when possible.

---

## Clipboard Security

When the user copies a code:

1. Code is written to system clipboard
2. A timer starts (configurable, default 30 seconds)
3. After timeout, clipboard is cleared (only if the clipboard still contains the copied code — do not clear user's other clipboard content)
4. On code refresh (period expires), the clipboard copy of the old code is cleared immediately

### Per-Platform Clipboard Behavior

| Platform | Clipboard Access | Auto-Clear |
|----------|-----------------|------------|
| Desktop (Tauri) | `tauri-plugin-clipboard-manager` | Yes, via Rust timer |
| Mobile (Tauri) | Same plugin | Yes, via Rust timer |
| Android 13+ | System auto-clears sensitive clipboard after 60s | Yes, native + app timer |
| iOS 16+ | Paste requires user confirmation from other apps | Yes, app timer |
| Chrome Extension | `navigator.clipboard` API | Yes, via alarm API |

---

## QR Code Security

QR codes contain `otpauth://` URIs with the raw secret. Handling:

1. `[MOBILE]` Camera feed is processed locally, never transmitted
2. `[DESKTOP]` Screen capture / image import is processed locally
3. `[EXTENSION]` Tab screenshot is captured via `chrome.tabs.captureVisibleTab`, processed in extension memory, screenshot immediately discarded
4. After QR decode, the raw URI is passed to the parser, the secret is extracted and stored in the vault, and ALL intermediate buffers (image data, URI string) are discarded
5. QR codes MUST NOT be cached, logged, or stored in any form

---

## Import Security

When importing from other authenticators:

1. The import file is read into memory
2. Parsed according to the source format
3. Each token is added to the vault (secrets encrypted)
4. The original file is NOT deleted (user's responsibility) but a warning SHOULD be shown: "Your import file contains unencrypted secrets. Consider deleting it."
5. Import buffer is cleared from memory after processing

### Supported Import Formats

| Source | Format | Notes |
|--------|--------|-------|
| Google Authenticator | `otpauth-migration://` protobuf | Decode protobuf, extract otpauth URIs |
| Aegis | JSON (encrypted or plain) | If encrypted, prompt for Aegis password |
| 2FAS | JSON | Parse 2FAS backup format |
| Ente Auth | JSON | Parse Ente export format |
| Plain text | List of `otpauth://` URIs | One per line |
| QR code image | Image file containing QR code | Scan and parse |

---

## Content Security Policy (Chrome Extension)

The extension MUST have a strict CSP:

- No `eval()`, no `unsafe-inline`, no `unsafe-eval`
- No remote script loading
- No WebAssembly (Manifest V3 restriction in service workers)
- `connect-src 'none'` in Phase 1 (no network access at all)
- All crypto via Web Crypto API (SubtleCrypto), which is built into the browser

---

## Audit Trail

Phase 1 does not include audit logging. However, the vault schema SHOULD include a `updated_at` timestamp on every token so that the user can see when a token was last modified. Full audit logging (who changed what, when, from which device) is a Phase 3 feature tied to sync.

---

## Security Checklist for Implementers

Before shipping, verify:

- [ ] Vault file is unreadable without master password (try opening with `sqlite3` — should fail)
- [ ] Changing one byte in the vault file causes open to fail (integrity check)
- [ ] Master password is never logged, stored, or transmitted
- [ ] Derived keys are zeroized on vault lock (verify with memory dump if possible)
- [ ] TOTP codes match reference implementations (use RFC 6238 test vectors)
- [ ] Clipboard auto-clear works on all platforms
- [ ] QR scanner does not retain image data after scan
- [ ] Chrome extension makes zero network requests (verify in DevTools)
- [ ] Extension CSP blocks all remote resource loading
- [ ] Biometric unlock stores key in platform keychain, not in app storage
- [ ] Import file warning is shown
- [ ] No secrets in log output (search all log statements)
- [ ] Argon2id parameters produce ~500ms unlock time on target hardware
- [ ] SQLCipher database file passes randomness tests (no plaintext patterns)
