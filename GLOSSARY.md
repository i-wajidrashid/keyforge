# Glossary

How to read the KeyForge spec documents, and what every term means.

---

## How to Read These Docs

### Document Hierarchy

```
README.md           ← Start here. Project overview, structure, phases.
GLOSSARY.md         ← You are here. Read this second.
ARCHITECTURE.md     ← How the codebase is organized and why.
SECURITY.md         ← How secrets are protected. Read before touching crypto.
UI-SPEC.md          ← Every screen, every interaction, every animation.
PLATFORM-SPEC.md    ← What changes per platform (desktop vs mobile vs extension).
TESTING.md          ← How to test, what to test, coverage targets.
PHASE-1-SPEC.md     ← What to build NOW. Acceptance criteria for Phase 1.
FUTURE-PHASES.md    ← What comes later. Phase 2, 3, 4. Do NOT build yet.
LANDING-PAGE.md     ← Marketing site spec. Separate from the app.
```

### Reading Order for Implementers

1. **README.md** — Understand the project
2. **GLOSSARY.md** — Learn the language
3. **ARCHITECTURE.md** — Understand the monorepo and tech stack
4. **SECURITY.md** — Understand how secrets are handled (read FULLY before writing any crypto or vault code)
5. **PHASE-1-SPEC.md** — Understand what to build
6. **UI-SPEC.md** — Understand what it looks like
7. **PLATFORM-SPEC.md** — Understand platform differences
8. **TESTING.md** — Understand how to validate
9. **FUTURE-PHASES.md** — Skim only. Understand what is coming so you do not paint yourself into a corner. Do not build any of this yet.

### Conventions Used in Specs

#### Requirement Levels

- **MUST** — Non-negotiable. The feature does not ship without this.
- **SHOULD** — Strongly recommended. Skip only with documented justification.
- **MAY** — Optional. Nice to have. Build if time permits.
- **MUST NOT** — Forbidden. Do not implement this. If you find yourself doing it, stop and re-read the spec.

#### Platform Tags

When a requirement is platform-specific, it is tagged:

- `[DESKTOP]` — Windows, macOS, Linux (Tauri desktop)
- `[MOBILE]` — Android, iOS (Tauri mobile)
- `[ANDROID]` — Android only
- `[iOS]` — iOS only
- `[EXTENSION]` — Chrome extension (Manifest V3)
- `[WEB]` — Web app (Phase 2, do not build yet)
- `[ALL]` — Every platform

#### Phase Tags

- `[P1]` — Phase 1 (build now)
- `[P2]` — Phase 2 (web app + minimal server, do not build yet)
- `[P3]` — Phase 3 (P2P sync, do not build yet)
- `[P4]` — Phase 4 (backup + recovery, do not build yet)

#### Layer Tags

- `[RUST]` — Implemented in Rust crate(s)
- `[TS]` — Implemented in TypeScript shared package(s)
- `[UI]` — Implemented in the shared UI layer
- `[SHELL]` — Implemented in the platform-specific app shell
- `[NATIVE]` — Requires platform-native API (via Tauri plugin or native bridge)

---

## Terms

### Core Concepts

**TOTP** — Time-Based One-Time Password. Defined in RFC 6238. Generates a code that changes every N seconds (default 30). This is what 99% of services use for 2FA.

**HOTP** — HMAC-Based One-Time Password. Defined in RFC 4226. Generates a code based on a counter that increments on each use. Older standard, some services still use it.

**OTP** — One-Time Password. Generic term covering both TOTP and HOTP.

**2FA** — Two-Factor Authentication. Using something you know (password) plus something you have (authenticator code) to log in.

**Secret** — The shared key between you and the service. Base32-encoded string. This is what gets stored in the vault. This is what generates the codes. Protect it at all costs.

**Token** — A single entry in the vault. Contains the secret, issuer, account name, algorithm, digits, and period. One token = one service's 2FA.

**Code** — The 6-8 digit number displayed to the user. Generated from the secret + current time (TOTP) or counter (HOTP). Ephemeral. Changes every period.

**Period** — The time window (in seconds) for which a TOTP code is valid. Default is 30 seconds. Some services use 60.

**Issuer** — The service name (e.g., "GitHub", "Google", "AWS"). Displayed alongside the code so users know which service it belongs to.

**Account** — The account identifier (e.g., "user@example.com"). Combined with issuer to uniquely identify a token.

### URI and Formats

**otpauth:// URI** — The standard URI format for sharing OTP secrets. Used in QR codes. Format: `otpauth://totp/Issuer:Account?secret=BASE32SECRET&issuer=Issuer&algorithm=SHA1&digits=6&period=30`

**QR Code** — A 2D barcode encoding an otpauth:// URI. Scanned to add a new token. This is how most services let you set up 2FA.

**Base32** — The encoding format for secrets in otpauth URIs. Not Base64. Uses characters A-Z and 2-7.

### Vault and Storage

**Vault** — The encrypted database that stores all tokens. One vault per device. Encrypted at rest using SQLCipher.

**Master Password** — The password that unlocks the vault. Used to derive the encryption key. Never stored anywhere.

**Key Derivation** — The process of turning a master password into an encryption key. Uses Argon2id. Makes brute-force attacks computationally expensive.

**SQLCipher** — A fork of SQLite that adds AES-256 encryption. The vault database engine.

**Keychain / Keystore** — Platform-native secure storage. macOS Keychain, Windows Credential Manager, Android Keystore, iOS Keychain. Used to store the derived vault key so biometric unlock works without re-entering the master password.

### Cryptography

**AES-256** — Advanced Encryption Standard with 256-bit keys. The encryption algorithm used by SQLCipher for the vault.

**Argon2id** — A memory-hard password hashing algorithm. Used for key derivation. Resistant to GPU and ASIC attacks.

**HMAC** — Hash-based Message Authentication Code. The core primitive in both TOTP and HOTP. Combines a secret key with a message using a hash function.

**SHA-1 / SHA-256 / SHA-512** — Hash functions used with HMAC in TOTP/HOTP. SHA-1 is the default (for compatibility). SHA-256 and SHA-512 are supported for services that require them.

### Platform

**Tauri v2** — The framework used to build desktop and mobile apps. Rust backend + web frontend. Not Electron. Much smaller binary, much less memory.

**Manifest V3** — The Chrome extension manifest format. Required for Chrome Web Store submission. Replaces persistent background pages with service workers.

**Service Worker** — The background script for a Manifest V3 Chrome extension. Runs on-demand, not persistently. Cannot use WebAssembly (relevant for crypto).

**Shell** — The platform-specific wrapper app. Contains Tauri config, platform permissions, native plugin bindings. Minimal logic — delegates everything to shared packages and Rust crates.

**Deep Link** — A URI that opens the app directly. Used for `otpauth://` URIs so clicking an OTP setup link opens KeyForge.

### Sync (Phase 3)

**P2P** — Peer-to-peer. Direct device-to-device communication without a central server.

**CRDT** — Conflict-free Replicated Data Type. A data structure that can be independently modified on multiple devices and merged without conflicts.

**Pairing** — The one-time process of linking two devices. Involves exchanging a shared secret (via QR code or manual entry).

**Discovery** — How two paired devices find each other on the network. Can be local (mDNS on same network) or remote (via a thin relay server).

### UI

**Design System** — The set of colors, typography, spacing, and components shared across all platforms. Defined in UI-SPEC.md.

**Surface** — A background layer in the UI. Surfaces have elevation levels (base, raised, overlay) that determine their brightness in dark mode.

**Toast** — A small non-blocking notification that appears briefly. Used for confirmations like "Code copied."

**Progress Ring** — The circular countdown indicator showing how much time is left before the current TOTP code expires.

**Compact Mode** — A reduced-height display mode for the code list. Shows more codes at once with less visual detail.

### Testing

**Unit Test** — Tests a single function or module in isolation. All Rust crates MUST have unit tests.

**Integration Test** — Tests multiple modules working together. Vault operations, TOTP generation from stored secrets, etc.

**E2E Test** — End-to-end test. Runs the actual app and simulates user interactions. Verifies full flows.

**Coverage** — The percentage of code lines/branches exercised by tests. Target: 100% for Rust crates, 90%+ for TypeScript packages.

**Property-Based Testing** — Generates random inputs to find edge cases. Used for TOTP/HOTP validation against reference implementations.

---

## Abbreviations

| Abbr | Meaning |
|------|---------|
| 2FA | Two-Factor Authentication |
| AES | Advanced Encryption Standard |
| CRDT | Conflict-free Replicated Data Type |
| E2E | End-to-End (testing or encryption, context-dependent) |
| HMAC | Hash-based Message Authentication Code |
| HOTP | HMAC-Based One-Time Password |
| KDF | Key Derivation Function |
| MV3 | Manifest Version 3 (Chrome extensions) |
| OTP | One-Time Password |
| P2P | Peer-to-Peer |
| QR | Quick Response (code) |
| RFC | Request for Comments (internet standard) |
| TOTP | Time-Based One-Time Password |
| URI | Uniform Resource Identifier |
