# KeyForge

**Free, open-source, local-first authenticator.**

TOTP/HOTP 2FA across desktop, mobile, browser extension, and web — all from one codebase. Zero tracking. Zero cloud. Your keys, your devices, your rules. P2P sync coming soon.

---

## What Is This

KeyForge is an authenticator app. It generates time-based (TOTP) and counter-based (HOTP) one-time passwords for two-factor authentication. It stores your secrets locally in an encrypted vault on your device. Nothing leaves your machine unless you explicitly tell it to.

It is not a password manager. It does one thing and does it right.

## Why Build Another One

Every existing authenticator is either:

- **Closed source** with telemetry baked in (Google Authenticator, Microsoft Authenticator, Authy)
- **Open source but single-platform** (Aegis is Android-only)
- **Cross-platform but requires an account** (Ente Auth, Proton Authenticator force cloud sync)
- **A browser extension with no mobile** or vice versa

KeyForge is all platforms, one codebase, local-first, no account required, no telemetry, no bullshit.

## Platforms

| Platform | Status | Runtime |
|----------|--------|---------|
| Windows | Phase 1 | Tauri v2 (desktop) |
| macOS | Phase 1 | Tauri v2 (desktop) |
| Linux | Phase 1 | Tauri v2 (desktop) |
| Android | Phase 1 | Tauri v2 (mobile) |
| iOS | Phase 1 | Tauri v2 (mobile) |
| Chrome Extension | Phase 1 | Manifest V3 |
| Web App | Phase 2 | Standalone SPA |

## Monorepo Structure

```
keyforge/
├── apps/
│   ├── desktop/          # Tauri v2 desktop shell (Win/Mac/Linux)
│   ├── mobile/           # Tauri v2 mobile shell (Android/iOS)
│   ├── extension/        # Chrome extension (Manifest V3)
│   └── web/              # Web app (Phase 2)
├── packages/
│   ├── ui/               # Shared UI components
│   ├── core/             # TOTP/HOTP logic, vault operations (TypeScript)
│   └── shared/           # Types, constants, utilities
├── crates/
│   ├── keyforge-crypto/  # Encryption, key derivation, TOTP/HOTP (Rust)
│   ├── keyforge-vault/   # Vault read/write, SQLCipher operations (Rust)
│   └── keyforge-sync/    # P2P sync engine (Rust, Phase 3)
├── sites/
│   └── landing/          # Marketing / landing page
└── docs/                 # Extended documentation
```

## Phases

### Phase 1 — Local Fortress (Current)

Desktop apps, mobile apps, Chrome extension. Everything local. Encrypted vault. Biometric unlock. QR code scanning. Import/export. No network calls.

See: [PHASE-1-SPEC.md](./PHASE-1-SPEC.md)

### Phase 2 — Web App + Minimal Server

Standalone web app. Thin server for optional account creation and encrypted blob storage only. Server never sees plaintext secrets. Client-side encryption before anything touches the wire.

### Phase 3 — P2P Sync

Device-to-device sync with no server in the middle. Encrypted CRDT-based state replication. Devices discover each other, pair once, sync forever. Server only assists with discovery (optional).

### Phase 4 — Backup + Recovery

Encrypted backup to user-chosen storage (local file, USB, cloud provider). Recovery flows. Emergency access. Paper backup codes.

See: [FUTURE-PHASES.md](./FUTURE-PHASES.md) for Phase 2, 3, 4 details.

## Spec Documents

| Document | What It Covers |
|----------|---------------|
| [GLOSSARY.md](./GLOSSARY.md) | Terminology and how to read these docs |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Monorepo structure, tech stack, shared packages, Rust crates |
| [SECURITY.md](./SECURITY.md) | Encryption, vault format, key derivation, threat model |
| [UI-SPEC.md](./UI-SPEC.md) | Design system, screens, interactions, animations |
| [PLATFORM-SPEC.md](./PLATFORM-SPEC.md) | Per-platform details (desktop, mobile, extension) |
| [TESTING.md](./TESTING.md) | Test strategy, Rust tests, E2E, coverage targets |
| [PHASE-1-SPEC.md](./PHASE-1-SPEC.md) | Phase 1 detailed requirements and acceptance criteria |
| [FUTURE-PHASES.md](./FUTURE-PHASES.md) | Phase 2, 3, 4 specs |
| [CI-CD.md](./CI-CD.md) | GitHub Actions workflows, runners, secrets, caching, release automation |
| [LANDING-PAGE.md](./LANDING-PAGE.md) | Marketing site spec |

## Core Principles

1. **Local-first** — Data lives on your device. Period.
2. **Zero knowledge** — When sync exists, the transport layer never sees plaintext.
3. **Zero telemetry** — No analytics, no tracking, no phone-home. Not even crash reporting unless the user opts in.
4. **Minimal surface** — Small binary, small attack surface, small dependency tree.
5. **Rust where it matters** — All cryptography, vault operations, and sync logic in Rust. No JS crypto.
6. **One codebase** — Shared UI, shared logic, platform-specific shells only where unavoidable.
7. **Test everything** — 100% Rust test coverage target. E2E tests for every platform. No exceptions.

## Standards Implemented

- RFC 6238 — TOTP: Time-Based One-Time Password Algorithm
- RFC 4226 — HOTP: HMAC-Based One-Time Password Algorithm
- otpauth:// URI scheme (Google Authenticator compatible)

## License

Open source. License TBD (likely MIT or AGPLv3 — to be decided before first public release).
