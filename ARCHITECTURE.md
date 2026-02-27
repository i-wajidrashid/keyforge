# Architecture

How the KeyForge monorepo is structured, what each piece does, and why.

---

## Monorepo Layout

```
keyforge/
│
├── apps/
│   ├── desktop/                  # Tauri v2 desktop app
│   │   ├── src-tauri/            # Rust backend (Cargo.toml, main.rs, commands)
│   │   ├── src/                  # Frontend entry point (imports from packages/ui)
│   │   ├── tauri.conf.json       # Tauri config (window size, permissions, capabilities)
│   │   └── capabilities/         # Tauri v2 capability files (per-platform permissions)
│   │
│   ├── mobile/                   # Tauri v2 mobile app
│   │   ├── src-tauri/            # Rust backend (same crate deps as desktop, mobile-specific config)
│   │   ├── src/                  # Frontend entry point (imports from packages/ui)
│   │   ├── tauri.conf.json       # Mobile-specific Tauri config
│   │   ├── gen/
│   │   │   ├── android/          # Generated Android project (Gradle, manifests)
│   │   │   └── apple/            # Generated iOS project (Xcode, Info.plist)
│   │   └── capabilities/
│   │
│   ├── extension/                # Chrome extension (Manifest V3)
│   │   ├── manifest.json         # MV3 manifest
│   │   ├── service-worker/       # Background service worker
│   │   ├── popup/                # Extension popup UI (imports from packages/ui)
│   │   ├── options/              # Extension options page
│   │   └── content-scripts/      # Content scripts for QR detection on web pages
│   │
│   └── web/                      # Web app [P2] (do not build yet)
│
├── packages/
│   ├── ui/                       # Shared UI components
│   │   ├── components/           # All reusable components
│   │   ├── screens/              # Full screen layouts
│   │   ├── hooks/                # Shared hooks (useCountdown, useVault, etc.)
│   │   ├── styles/               # Design system tokens, global styles
│   │   └── index.ts              # Public API barrel export
│   │
│   ├── core/                     # Shared business logic (TypeScript)
│   │   ├── totp/                 # TOTP generation/validation (wraps Rust via Tauri commands, pure JS for extension)
│   │   ├── hotp/                 # HOTP generation/validation
│   │   ├── uri/                  # otpauth:// URI parsing and generation
│   │   ├── vault/                # Vault CRUD operations (abstracts over platform storage)
│   │   └── index.ts
│   │
│   └── shared/                   # Shared types, constants, utilities
│       ├── types/                # TypeScript type definitions
│       ├── constants/            # App-wide constants (defaults, limits, etc.)
│       └── utils/                # Pure utility functions (formatCode, timeLeft, etc.)
│
├── crates/
│   ├── keyforge-crypto/          # Rust: ALL cryptography lives here
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── totp.rs           # TOTP implementation (RFC 6238)
│   │   │   ├── hotp.rs           # HOTP implementation (RFC 4226)
│   │   │   ├── kdf.rs            # Key derivation (Argon2id)
│   │   │   ├── aead.rs           # Authenticated encryption (AES-256-GCM)
│   │   │   └── random.rs         # Secure random number generation
│   │   ├── tests/                # Unit + integration tests
│   │   ├── benches/              # Benchmarks (Argon2id tuning, TOTP throughput)
│   │   └── Cargo.toml
│   │
│   ├── keyforge-vault/           # Rust: Vault database operations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs             # SQLCipher connection management
│   │   │   ├── migrations.rs     # Schema migrations (versioned)
│   │   │   ├── token.rs          # Token CRUD (insert, read, update, delete, reorder)
│   │   │   ├── export.rs         # Export vault to encrypted file
│   │   │   └── import.rs         # Import from other authenticators
│   │   ├── tests/
│   │   └── Cargo.toml
│   │
│   └── keyforge-sync/            # Rust: P2P sync [P3] (do not build yet, create empty crate with README)
│       ├── src/
│       │   └── lib.rs            # Placeholder
│       └── Cargo.toml
│
├── sites/
│   └── landing/                  # Marketing landing page
│       ├── src/                  # Page source
│       ├── public/               # Static assets (screenshots, icons)
│       └── ...
│
├── tools/                        # Build scripts, CI helpers
│   ├── scripts/                  # Monorepo-level scripts
│   └── ci/                       # CI/CD pipeline configs
│
├── Cargo.toml                    # Workspace-level Cargo.toml (all crates)
├── pnpm-workspace.yaml           # pnpm workspace config
├── turbo.json                    # Turborepo pipeline config
├── .github/                      # GitHub Actions workflows
└── .gitignore
```

---

## Tech Stack Decisions

### Why Tauri v2 (Not Electron, Not React Native, Not Flutter)

| Factor | Tauri v2 | Electron | React Native | Flutter |
|--------|----------|----------|-------------|---------|
| Binary size | ~5-10 MB | ~150+ MB | N/A (no desktop) | ~20 MB |
| Memory usage | Low (system webview) | High (bundled Chromium) | Medium | Medium |
| Rust backend | Native | Requires FFI | Requires bridge | Requires FFI |
| Desktop + Mobile | Yes (v2) | Desktop only | Mobile only (Expo has desktop but fragile) | Both (but different paradigm) |
| Crypto in Rust | Direct | Needs native module | Needs native module | Needs FFI |
| System webview | Uses OS webview | Bundles Chromium | Own renderer | Own renderer |

Tauri v2 gives us Rust-native crypto, tiny binaries, and desktop + mobile from one project. The tradeoff is that mobile support is newer and the ecosystem is smaller. Worth it for this use case.

### Why Rust for Crypto and Vault

- TOTP/HOTP, key derivation, encryption — all MUST be in Rust. No JavaScript crypto libraries. Period.
- Rust's memory safety guarantees prevent entire classes of vulnerabilities (buffer overflows, use-after-free).
- The `ring` or `RustCrypto` ecosystem provides audited, production-grade crypto primitives.
- SQLCipher via `rusqlite` with `bundled-sqlcipher` feature gives us encrypted SQLite with zero system dependencies.

### Why NOT Stronghold

Tauri's Stronghold plugin is being deprecated in Tauri v3. Do not use it. Use SQLCipher directly via rusqlite.

### Frontend Framework

The UI framework for `packages/ui` MUST be chosen based on:

1. Works in Tauri webview (desktop + mobile)
2. Works in Chrome extension popup
3. Small bundle size (extension popup budget is tiny)
4. Good component model

Recommended: **SolidJS** or **Svelte**. Both are small, fast, compile away the framework overhead, and work in all targets. React works too but is heavier. The implementer should choose one and commit — do not mix frameworks.

Whatever is chosen, all UI components live in `packages/ui` and are consumed by every app shell.

### Why pnpm + Turborepo

- pnpm workspaces for linking packages within the monorepo
- Turborepo for build orchestration, caching, and parallel task execution
- Both are industry standard for monorepo management
- Turborepo handles the dependency graph between packages so `packages/ui` rebuilds when `packages/shared` changes

### SQLCipher via rusqlite

The vault database uses `rusqlite` with the `bundled-sqlcipher` feature:

- `bundled-sqlcipher` compiles SQLCipher from source into the binary — no system library dependency
- `bundled-sqlcipher-vendored-openssl` vendors OpenSSL too, so the entire crypto stack is self-contained
- This means the vault encryption works identically on every platform with zero external dependencies

---

## Shared Package Boundaries

### packages/core

This is the TypeScript business logic layer. It wraps the Rust crates via Tauri commands (on desktop/mobile) and provides pure TypeScript fallbacks (for the Chrome extension, which cannot call Rust).

```
packages/core/
├── totp/
│   ├── generate.ts       # Calls Rust on Tauri, falls back to JS on extension
│   ├── validate.ts       # Check if a given code is valid for a secret
│   └── countdown.ts      # Time remaining until next code
├── hotp/
│   ├── generate.ts
│   └── validate.ts
├── uri/
│   ├── parse.ts          # Parse otpauth:// URI into Token object
│   └── encode.ts         # Encode Token object into otpauth:// URI
├── vault/
│   ├── adapter.ts        # Abstract storage interface
│   ├── tauri-adapter.ts  # Tauri command-based adapter (desktop/mobile)
│   └── extension-adapter.ts  # chrome.storage-based adapter (extension)
└── types.ts              # Re-exports from packages/shared
```

**Critical rule:** The Chrome extension CANNOT call Rust. The extension MUST have a pure TypeScript implementation of TOTP/HOTP that uses the Web Crypto API (SubtleCrypto). This is because:

1. Chrome Manifest V3 service workers do not support WebAssembly
2. There is no Rust bridge in a browser extension
3. The Web Crypto API provides HMAC-SHA1/256/512 natively

The `packages/core` functions MUST detect their runtime environment and dispatch to either Rust (via Tauri invoke) or pure TypeScript (via Web Crypto API) transparently.

### packages/ui

All visual components. Organized by function:

```
packages/ui/
├── components/
│   ├── CodeCard/          # Single token display with code, progress ring, copy
│   ├── CodeList/          # Scrollable list of CodeCards
│   ├── SearchBar/         # Filter tokens by issuer/account
│   ├── AddToken/          # Add token flow (QR scan, manual entry, URI paste)
│   ├── Settings/          # Settings panel
│   ├── VaultLock/         # Lock screen (master password / biometric)
│   ├── ProgressRing/      # Circular countdown timer
│   ├── Toast/             # Non-blocking notification
│   ├── Modal/             # Confirmation dialogs
│   ├── EmptyState/        # Shown when vault has no tokens
│   └── Icons/             # Custom icon set (no AI-looking icons)
├── screens/
│   ├── HomeScreen/        # Main screen: code list
│   ├── LockScreen/        # Vault unlock
│   ├── AddScreen/         # Add token flow
│   ├── SettingsScreen/    # Settings
│   ├── TokenDetailScreen/ # Single token detail/edit
│   └── OnboardingScreen/  # First launch setup
├── hooks/
│   ├── useCountdown.ts    # Seconds remaining for current TOTP period
│   ├── useVault.ts        # Vault state management
│   ├── useSearch.ts       # Token search/filter
│   ├── useCopy.ts         # Copy to clipboard with auto-clear
│   └── useBiometric.ts    # Biometric availability and trigger
├── styles/
│   ├── tokens.css         # Design system CSS custom properties
│   ├── reset.css          # CSS reset / normalize
│   └── animations.css     # Shared keyframe animations
└── index.ts
```

### packages/shared

Pure types and utilities. Zero runtime dependencies. Zero side effects.

```
packages/shared/
├── types/
│   ├── token.ts           # Token type definition
│   ├── vault.ts           # Vault metadata types
│   ├── settings.ts        # User settings types
│   └── platform.ts        # Platform detection types
├── constants/
│   ├── defaults.ts        # Default TOTP period (30s), default digits (6), etc.
│   ├── limits.ts          # Max tokens, max issuer length, etc.
│   └── algorithms.ts      # Supported algorithms enum (SHA1, SHA256, SHA512)
└── utils/
    ├── formatCode.ts      # "123456" → "123 456" (with space in middle)
    ├── timeLeft.ts        # Seconds until next period boundary
    ├── base32.ts          # Base32 encode/decode (for URI handling)
    └── platform.ts        # Detect current platform (tauri-desktop, tauri-mobile, extension, web)
```

---

## Rust Crate Boundaries

### keyforge-crypto

This crate MUST:

- Implement TOTP (RFC 6238) with HMAC-SHA1, HMAC-SHA256, HMAC-SHA512
- Implement HOTP (RFC 4226) with the same hash algorithms
- Implement key derivation using Argon2id (configurable memory, time, and parallelism params)
- Implement AES-256-GCM for authenticated encryption (used by vault and future export)
- Provide secure random byte generation (for salt, nonce generation)
- Have ZERO dependencies on Tauri (this is a pure Rust crate, usable outside Tauri)
- Be fully tested with 100% line coverage
- Include property-based tests that validate against RFC test vectors

### keyforge-vault

This crate MUST:

- Depend on `keyforge-crypto` for all crypto operations
- Manage SQLCipher database connections (open, close, key rotation)
- Run versioned schema migrations on first open and on upgrade
- Provide CRUD operations for tokens (insert, read all, read one, update, delete, reorder)
- Handle vault locking (zeroize the key from memory on lock)
- Provide import functions (from Google Authenticator, Aegis, 2FAS, Ente Auth, plain otpauth URIs)
- Provide export functions (encrypted file, plaintext otpauth URIs for migration)
- Have ZERO dependencies on Tauri
- Be fully tested

### keyforge-sync (Phase 3 — placeholder only)

Create the crate with an empty `lib.rs` and a `README.md` inside explaining it will contain:

- P2P device discovery (mDNS for local network, optional relay for remote)
- CRDT-based token list synchronization
- Encrypted transport (Noise protocol or TLS 1.3)
- Pairing flow (QR code or manual code exchange)

Do NOT implement anything in this crate yet.

---

## Tauri Command Bridge

The Tauri apps (desktop and mobile) expose Rust functions to the frontend via Tauri commands. These commands live in `apps/desktop/src-tauri/` and `apps/mobile/src-tauri/` and are thin wrappers around the Rust crates.

Command categories:

| Command Group | Examples |
|--------------|----------|
| `vault_*` | `vault_unlock`, `vault_lock`, `vault_is_locked`, `vault_create` |
| `token_*` | `token_list`, `token_add`, `token_delete`, `token_update`, `token_reorder` |
| `otp_*` | `otp_generate_totp`, `otp_generate_hotp`, `otp_validate` |
| `crypto_*` | `crypto_derive_key`, `crypto_encrypt`, `crypto_decrypt` |
| `scan_*` | `scan_qr_from_camera`, `scan_qr_from_image` |
| `platform_*` | `platform_biometric_available`, `platform_biometric_authenticate`, `platform_keychain_store`, `platform_keychain_read` |

Every command MUST:

- Return a `Result<T, String>` (Tauri's error convention)
- Log errors (not secrets) for debugging
- Never log or expose secret key material

---

## Data Flow

### TOTP Code Generation (Desktop/Mobile)

```
User sees code → UI calls useCountdown hook
  → hook calls packages/core/totp/generate
    → core detects Tauri runtime
      → calls Tauri command otp_generate_totp(secret, algorithm, digits, period)
        → Tauri invokes keyforge-crypto::totp::generate()
          → HMAC(secret, time_counter) → truncate → return code
        → code returned to frontend
      → UI displays code
```

### TOTP Code Generation (Chrome Extension)

```
User sees code → UI calls useCountdown hook
  → hook calls packages/core/totp/generate
    → core detects extension runtime (no Tauri)
      → calls pure TypeScript TOTP using Web Crypto API
        → SubtleCrypto.sign("HMAC", key, counter) → truncate → return code
      → UI displays code
```

### Token Addition (QR Scan)

```
User taps "Add" → AddScreen shown
  → User chooses "Scan QR"
    → [MOBILE] Camera opens via Tauri barcode scanner plugin
    → [DESKTOP] Screen region capture or file picker
    → [EXTENSION] Captures current tab screenshot, scans for QR
  → QR decoded → otpauth:// URI extracted
    → packages/core/uri/parse processes URI
      → Token object created
        → Tauri command token_add called (or chrome.storage on extension)
          → keyforge-vault inserts into SQLCipher
        → UI updates code list
        → Toast: "Token added"
```

---

## Build Pipeline

### Development

```
pnpm dev              # Starts all dev servers
pnpm dev:desktop      # Tauri desktop dev (hot reload)
pnpm dev:mobile       # Tauri mobile dev (Android/iOS emulator)
pnpm dev:extension    # Extension dev (watch + rebuild, load unpacked in Chrome)
pnpm dev:landing      # Landing page dev server
```

### Testing

```
pnpm test             # Run all tests
pnpm test:rust        # cargo test --workspace (all Rust crates)
pnpm test:ts          # Run TypeScript unit tests
pnpm test:e2e         # Run E2E tests per platform
pnpm test:coverage    # Generate coverage reports
```

### Building

```
pnpm build            # Build everything
pnpm build:desktop    # Tauri desktop build (produces installers)
pnpm build:mobile     # Tauri mobile build (APK + IPA)
pnpm build:extension  # Extension build (produces zip for Chrome Web Store)
pnpm build:landing    # Landing page static build
```

---

## Dependency Rules

1. `packages/shared` depends on NOTHING
2. `packages/core` depends on `packages/shared`
3. `packages/ui` depends on `packages/shared` and `packages/core`
4. `apps/*` depend on `packages/ui` (and transitively on core and shared)
5. `sites/landing` depends on NOTHING from the app packages (it is standalone)
6. `crates/keyforge-vault` depends on `crates/keyforge-crypto`
7. `crates/keyforge-crypto` depends on NOTHING from the monorepo (only external Rust crates)
8. `crates/keyforge-sync` depends on `crates/keyforge-crypto` (Phase 3)

No circular dependencies. No upward dependencies. Clean DAG.

---

## Future-Proofing Decisions

These architectural choices are made NOW to avoid painful refactoring LATER:

1. **Vault adapter abstraction** — `packages/core/vault/adapter.ts` defines an interface. Desktop/mobile use the Tauri adapter. Extension uses chrome.storage adapter. Web (Phase 2) will use IndexedDB adapter. P2P sync (Phase 3) will add a sync layer on top of the adapter. Design the interface now, swap implementations later.

2. **Token type includes sync metadata fields** — The Token type SHOULD include optional fields for `lastModified`, `deviceId`, and `syncVersion` even in Phase 1. These fields are nullable and unused, but having them in the schema from day one prevents a migration headache in Phase 3.

3. **Command pattern for mutations** — All vault mutations (add, delete, update, reorder) SHOULD go through a command/event pattern. This makes it trivial to add a sync outbox in Phase 3 — every mutation generates an event that the sync engine can replicate.

4. **SQLCipher migration system** — The vault MUST have a versioned migration system from day one. Migrations are numbered and run in order. The vault stores its current schema version. This prevents data loss on app updates.
