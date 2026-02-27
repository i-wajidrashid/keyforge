# Future Phases

Phase 2, 3, and 4. Do NOT build any of this yet. This document exists so that Phase 1 architecture decisions do not block future work.

Read GLOSSARY.md for conventions before reading this document.

---

## Phase 2 — Web App + Minimal Server

### Goal

Ship a standalone web app (SPA) and introduce a minimal server component for optional encrypted blob storage. The server NEVER sees plaintext secrets.

### Web App

**What it is:**
- A Progressive Web App (PWA) accessible via browser
- Uses the same `packages/ui`, `packages/core`, `packages/shared` as other platforms
- Vault stored in IndexedDB (encrypted with AES-256-GCM, key derived from master password via PBKDF2)
- TOTP/HOTP via Web Crypto API (same as Chrome extension)

**What it is NOT:**
- Not a backend-rendered app
- Not a SaaS product
- Not required to use the server component

**Storage adapter:**
- New adapter: `packages/core/vault/indexeddb-adapter.ts`
- IndexedDB stores the same encrypted vault format as the Chrome extension (JSON blob, AES-256-GCM encrypted)
- PBKDF2 key derivation (same tradeoff as Chrome extension — Argon2id not available in browser)

**Platform differences:**
- No biometric unlock (browser has no keychain access)
- No QR scanning from camera by default (MAY use `navigator.mediaDevices.getUserMedia` with user permission)
- No deep linking (browser URL does not register for `otpauth://` scheme)
- No system tray, no autostart

### Minimal Server

**Architecture:**

The server is a thin, dumb, encrypted blob store. It stores opaque encrypted data. It cannot read, parse, or decrypt anything it stores.

**Server responsibilities:**
1. Account creation (email + password, or passkey)
2. Receive encrypted vault blob from client
3. Store encrypted vault blob
4. Return encrypted vault blob to authenticated client
5. That is it. No processing. No indexing. No analytics. No logs of vault content.

**Server does NOT:**
- Decrypt vault data (it does not have the key)
- Know how many tokens are in a vault
- Know which services the user has 2FA for
- Parse any vault content
- Log request bodies
- Store analytics or tracking data

**Protocol:**
- Client-side: encrypt entire vault state into a single blob using a key derived from master password
- Upload: `PUT /vault` with auth header + encrypted blob
- Download: `GET /vault` with auth header → encrypted blob
- Server stores the blob as-is (opaque bytes)
- Version/conflict: include a monotonic version number. Server rejects upload if version is not exactly current+1 (prevents lost updates)

**Auth:**
- Email/password account creation (server stores hashed password via Argon2id)
- The server password is SEPARATE from the vault master password (different credentials)
- Session tokens (JWT or opaque) for API auth
- Optional: passkey/WebAuthn support

**Tech stack for server:**
- Rust (Axum or Actix-web) — keep the stack consistent
- PostgreSQL or SQLite for account storage and blob metadata
- S3-compatible object storage for vault blobs (or just filesystem for small scale)
- Deployable as a single binary (self-hosted friendly)

**Self-hosting:**
- The server MUST be trivially self-hostable
- Single binary + config file + database
- Docker image provided
- Documentation for self-hosting on a VPS

### Phase 2 Web App Acceptance Criteria

- [ ] Web app accessible at a public URL
- [ ] Same UI as other platforms (responsive, works on mobile browsers too)
- [ ] Vault creation and unlock via master password
- [ ] TOTP/HOTP code generation via Web Crypto API
- [ ] Token CRUD (add, edit, delete, reorder)
- [ ] Manual entry and URI paste
- [ ] Import/export working
- [ ] Optional: sign up for account and sync vault blob to server
- [ ] Server stores ONLY encrypted blobs
- [ ] Server source code is open source and self-hostable
- [ ] No analytics, no tracking, no telemetry on server

---

## Phase 3 — P2P Sync

### Goal

Sync tokens between your own devices without relying on any server. Device-to-device. Encrypted in transit. Conflict-free.

### How It Works

**Pairing:**

1. Device A generates a pairing code (short alphanumeric code or QR code)
2. Device B enters or scans the pairing code
3. Both devices derive a shared secret from the pairing code using a key agreement protocol (X25519 Diffie-Hellman)
4. Devices are now "paired" — they can sync with each other
5. Pairing is permanent until explicitly revoked

**Discovery:**

How paired devices find each other on the network:

| Method | When | How |
|--------|------|-----|
| Local network (mDNS) | Same WiFi | Multicast DNS service advertisement |
| Relay server (optional) | Different networks | Thin relay that forwards encrypted packets between devices (server cannot read content) |
| Manual IP | Fallback | User enters device IP:port manually |

The relay server (if used) is even thinner than the Phase 2 server. It is a stateless packet forwarder. It stores nothing. It sees only encrypted bytes.

**Sync Protocol:**

- Each token mutation (add, edit, delete, reorder) generates a CRDT operation
- Operations are stored in a local outbox
- When two devices connect, they exchange operation logs
- CRDT merge resolves conflicts automatically (last-write-wins for updates, set-union for adds, tombstones for deletes)
- All operations are encrypted with the shared pairing secret before transmission
- Transport: Noise protocol over TCP (or WebRTC DataChannel for browser-to-device sync)

**CRDT Design:**

The token list is modeled as a CRDT using:

- **LWW-Register** (Last Writer Wins) for each token field (issuer, account, sort_order, etc.)
- **OR-Set** (Observed-Remove Set) for the token collection (handles add/remove conflicts)
- Each operation includes: device_id, logical timestamp (Lamport clock or hybrid logical clock), operation type, payload

This is why the `tokens` table in Phase 1 includes nullable `last_modified`, `device_id`, and `sync_version` fields — they become non-nullable in Phase 3.

**The `keyforge-sync` Rust crate will contain:**

- CRDT data structures and merge logic
- Operation log (append-only, encrypted at rest)
- Peer discovery (mDNS + optional relay)
- Encrypted transport (Noise protocol handshake + ChaCha20-Poly1305 stream cipher)
- Pairing flow (X25519 key agreement)
- Conflict resolution (automatic via CRDT, manual override for edge cases)

### Phase 3 Acceptance Criteria

- [ ] Two devices can pair via QR code or manual code
- [ ] Paired devices discover each other on local network (mDNS)
- [ ] Token changes on Device A appear on Device B within seconds (on same network)
- [ ] Sync works offline — changes queue up and sync when devices reconnect
- [ ] Conflicts are resolved automatically (CRDT merge)
- [ ] All sync traffic is end-to-end encrypted (pairing secret)
- [ ] Optional relay server enables sync across different networks
- [ ] Relay server cannot read sync data (encrypted)
- [ ] User can see list of paired devices and revoke pairing
- [ ] Revoking a device removes its ability to sync (re-pairing required)
- [ ] Sync works between any combination: desktop ↔ desktop, desktop ↔ mobile, mobile ↔ mobile, any ↔ extension
- [ ] Extension sync: uses WebRTC DataChannel or relay (extension cannot open raw TCP sockets)

### What Phase 1 Must Prepare

To avoid refactoring in Phase 3:

1. Token schema includes `last_modified`, `device_id`, `sync_version` (nullable)
2. Vault adapter interface supports a `subscribe(onChange)` method for change notifications
3. Token mutations go through a command/event pattern (not raw SQL)
4. Device ID is generated on first launch and stored in vault metadata

---

## Phase 4 — Backup + Recovery

### Goal

Provide robust backup and recovery options so users never lose access to their 2FA tokens.

### Backup Methods

| Method | Description |
|--------|-------------|
| Encrypted file | Export entire vault to a password-protected file. Store on USB, cloud drive, wherever. |
| Paper backup | Generate printable QR codes for each token (or a master QR that contains all tokens encrypted). |
| Cloud backup | Encrypted blob to user's cloud provider (Google Drive, iCloud, OneDrive, Dropbox). KeyForge provides the encryption, user provides the storage. |
| Scheduled auto-backup | Configurable: auto-export encrypted file to a specified location on a schedule (daily, weekly). Desktop only. |

### Recovery Flows

| Scenario | Recovery Path |
|----------|---------------|
| New device, have backup file | Install KeyForge → Import encrypted backup → enter backup password |
| New device, have synced device | Install KeyForge → Pair with existing device → sync |
| New device, have paper backup | Install KeyForge → Scan paper QR codes one by one |
| Lost master password | CANNOT recover. This is by design. Warn user during setup. |
| Lost all devices + no backup | CANNOT recover. This is by design. This is why backups are strongly encouraged. |

### Emergency Access (stretch)

- Designate a trusted contact
- Trusted contact can request access
- User has N days to deny the request
- If not denied, trusted contact receives the encrypted backup + a derived key
- This is complex and privacy-sensitive. Consider carefully before implementing.

### Phase 4 Acceptance Criteria

- [ ] Encrypted file backup: export and import works across all platforms
- [ ] Paper backup: generates printable PDF with QR codes per token
- [ ] Cloud backup: user authorizes their cloud storage, app uploads encrypted blob
- [ ] Scheduled auto-backup works on desktop
- [ ] Recovery from encrypted file works (new device + backup file → restored vault)
- [ ] Recovery via sync works (new device + paired device → synced vault)
- [ ] Lost master password: app clearly communicates this is unrecoverable
- [ ] Backup reminder: app prompts user to create a backup if they have not done so after 30 days

### What Phase 1 Must Prepare

1. Export to encrypted file already works in Phase 1 (this IS backup v0)
2. Import already works in Phase 1 (this IS recovery v0)
3. Vault file format is stable and versioned (so future backups are forward-compatible)

---

## Phase Summary

| Phase | Focus | Server Required | Network Required |
|-------|-------|----------------|-----------------|
| 1 | Local app, all platforms | No | No |
| 2 | Web app + optional cloud vault | Yes (minimal, encrypted blob store) | Yes (optional) |
| 3 | P2P sync between devices | Optional (relay for remote sync) | Yes (local or internet) |
| 4 | Backup and recovery | No (uses user's own storage) | Optional (for cloud backup) |

Each phase builds on the previous. No phase breaks what came before. A user who never goes online still has a fully functional app after Phase 1, forever.
