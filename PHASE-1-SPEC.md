# Phase 1 Spec — Local Fortress

Everything that MUST ship in Phase 1. Desktop (Win/Mac/Linux), Mobile (Android/iOS), Chrome Extension. All local. No network.

Read GLOSSARY.md for conventions (MUST/SHOULD/MAY, platform tags, etc.) before reading this document.

---

## Scope

Phase 1 delivers a fully functional, offline authenticator app across five platforms plus a Chrome extension. There are ZERO network calls. The app is a local vault that generates TOTP/HOTP codes.

### In Scope

- Vault creation with master password
- Vault encryption (SQLCipher + per-secret AES-256-GCM)
- TOTP code generation (SHA1, SHA256, SHA512)
- HOTP code generation (SHA1, SHA256, SHA512)
- QR code scanning (camera on mobile, screen/file on desktop, tab capture on extension)
- Manual token entry
- otpauth:// URI paste and parse
- Import from: Google Authenticator, Aegis, 2FAS, Ente Auth, plain text URI list
- Export to: encrypted KeyForge format, plain text URI list
- Biometric unlock (macOS Touch ID, Windows Hello, Android fingerprint/face, iOS Face ID/Touch ID)
- Clipboard copy with auto-clear
- Auto-lock with configurable timeout
- Search/filter tokens
- Reorder tokens (drag and drop)
- Edit token (issuer, account name)
- Delete token with confirmation
- Settings screen
- System tray (desktop)
- Deep linking for otpauth:// URIs (desktop, mobile)
- Dark theme (primary)
- Keyboard shortcuts (desktop)
- Landing page / marketing site

### Out of Scope (Phase 1)

- Light theme (MAY ship if time permits, not required)
- Web app (Phase 2)
- Any server component (Phase 2)
- Any sync between devices (Phase 3)
- Backup to external storage (Phase 4)
- Recovery flows (Phase 4)
- Internationalization / localization (future)
- Browser extensions for Firefox, Edge, Safari (future — Chrome only in Phase 1)

---

## Feature Specifications

### F1: Vault Creation `[ALL]` `[RUST]`

**Description:** On first app launch, create a new encrypted vault protected by a master password.

**Acceptance Criteria:**
- [ ] User enters a master password and confirmation
- [ ] Password strength indicator shows Weak / Fair / Strong / Very Strong
- [ ] Weak password shows warning but does NOT block creation
- [ ] Two Argon2id key derivations are performed (SQLCipher key + secret encryption key) with random salts
- [ ] SQLCipher database is created with the derived key
- [ ] Initial schema migration runs (creates `tokens`, `vault_meta`, `migrations` tables)
- [ ] Vault file is written to platform-appropriate storage location
- [ ] Salt and Argon2id parameters are stored in vault metadata
- [ ] On Chrome extension: vault is created in chrome.storage.local with PBKDF2 (Argon2id not available)
- [ ] Vault creation takes < 3 seconds on target hardware
- [ ] After creation, user is on the Home screen (empty state)

### F2: Vault Unlock `[ALL]` `[RUST]`

**Description:** Unlock the vault with master password or biometrics.

**Acceptance Criteria:**
- [ ] Password entry field is auto-focused on lock screen
- [ ] Show/hide password toggle works
- [ ] Correct password → derive key → open SQLCipher → navigate to Home screen
- [ ] Wrong password → shake animation → "Wrong password" error → clear input
- [ ] Key derivation shows loading indicator (does not freeze UI)
- [ ] `[DESKTOP macOS]` Touch ID prompt appears if configured
- [ ] `[DESKTOP Windows]` Windows Hello prompt appears if configured
- [ ] `[MOBILE Android]` Fingerprint/Face prompt appears if configured
- [ ] `[MOBILE iOS]` Face ID / Touch ID prompt appears if configured
- [ ] Biometric success → retrieve key from platform keychain → open vault
- [ ] Biometric failure → fall back to password entry
- [ ] `[EXTENSION]` Password only (no biometric option shown)
- [ ] Vault unlock takes < 1 second after key derivation completes

### F3: Biometric Setup `[DESKTOP macOS/Windows]` `[MOBILE]` `[NATIVE]`

**Description:** Enable biometric unlock during onboarding or later in settings.

**Acceptance Criteria:**
- [ ] During onboarding, if biometric hardware is detected, offer to enable biometric unlock
- [ ] User can enable/disable biometric in Settings at any time
- [ ] When enabled: store derived vault key in platform keychain/keystore with biometric access control
- [ ] When disabled: remove vault key from keychain/keystore
- [ ] If user re-enrolls biometrics (e.g., adds a new fingerprint), the keychain item SHOULD be invalidated on iOS (this is OS behavior with `kSecAccessControlBiometryCurrentSet`)
- [ ] If biometric hardware is not available, the biometric option is hidden (not shown as disabled)

### F4: TOTP Code Generation `[ALL]` `[RUST]` `[TS]`

**Description:** Generate TOTP codes per RFC 6238.

**Acceptance Criteria:**
- [ ] Supports SHA1, SHA256, SHA512 hash algorithms
- [ ] Supports 6-digit and 8-digit codes
- [ ] Supports configurable period (30s default, must also work with 60s and other values)
- [ ] Codes match RFC 6238 test vectors (see TESTING.md for exact values)
- [ ] `[DESKTOP]` `[MOBILE]` Codes are generated in Rust (keyforge-crypto crate)
- [ ] `[EXTENSION]` Codes are generated in TypeScript (Web Crypto API)
- [ ] Both implementations produce IDENTICAL output for the same inputs
- [ ] Code refreshes automatically when period expires (no user action required)
- [ ] Code display format: "123 456" (6-digit) or "1234 5678" (8-digit)

### F5: HOTP Code Generation `[ALL]` `[RUST]` `[TS]`

**Description:** Generate HOTP codes per RFC 4226.

**Acceptance Criteria:**
- [ ] Supports SHA1, SHA256, SHA512 hash algorithms
- [ ] Supports 6-digit and 8-digit codes
- [ ] Codes match RFC 4226 test vectors (see TESTING.md)
- [ ] Counter increments on code generation (user triggers, not automatic)
- [ ] Counter persists in vault
- [ ] HOTP tokens display a "Refresh" button instead of a progress ring
- [ ] Tapping refresh generates next code and increments counter

### F6: QR Code Scanning `[ALL]` `[NATIVE]`

**Description:** Scan QR codes to add tokens.

**Acceptance Criteria:**
- [ ] `[MOBILE]` Tapping "Scan QR" opens camera viewfinder
- [ ] `[MOBILE]` Camera permission requested on first use. Denied → show message with link to settings.
- [ ] `[MOBILE]` QR detection is real-time. On detect: haptic feedback, auto-parse URI.
- [ ] `[MOBILE]` Camera viewfinder closes automatically after successful scan
- [ ] `[DESKTOP]` User can select a screen region to scan for QR code
- [ ] `[DESKTOP]` User can import an image file containing a QR code
- [ ] `[EXTENSION]` "Scan QR" captures the current tab via `chrome.tabs.captureVisibleTab`
- [ ] `[EXTENSION]` If QR found in tab → parse and show confirmation
- [ ] `[EXTENSION]` If no QR found → show "No QR code found on this page"
- [ ] `[ALL]` Parsed `otpauth://` URI is shown in confirmation screen before adding
- [ ] `[ALL]` Invalid QR content (not otpauth URI) → show "Not a valid authenticator QR code"
- [ ] `[ALL]` Image data / camera feed is discarded immediately after scan (never stored)

### F7: Manual Token Entry `[ALL]` `[UI]`

**Description:** Add token by manually entering secret and metadata.

**Acceptance Criteria:**
- [ ] Form fields: Issuer (required), Account (optional), Secret Key (required)
- [ ] Advanced options (collapsed by default): Type (TOTP/HOTP), Algorithm, Digits, Period
- [ ] Secret key field accepts Base32 input (strips spaces and dashes automatically)
- [ ] Invalid Base32 → show validation error before submission
- [ ] Duplicate detection: if same issuer + account already exists, show warning (allow add anyway)
- [ ] On submit → encrypt secret → store in vault → navigate to Home → show Toast

### F8: otpauth:// URI Handling `[ALL]` `[TS]`

**Description:** Parse otpauth:// URIs from paste or deep link.

**Acceptance Criteria:**
- [ ] Paste field accepts `otpauth://totp/...` and `otpauth://hotp/...` URIs
- [ ] Parser extracts: type, issuer, account, secret, algorithm, digits, period, counter
- [ ] Missing issuer → extract from label (before colon) or set to "Unknown"
- [ ] Missing algorithm → default SHA1
- [ ] Missing digits → default 6
- [ ] Missing period → default 30
- [ ] Invalid URI → show "Invalid otpauth URI" error with details
- [ ] `[DESKTOP]` `[MOBILE]` Deep link: clicking `otpauth://` URI in browser opens KeyForge with the parsed token confirmation
- [ ] `[EXTENSION]` Deep linking not applicable (extension processes URIs internally)

### F9: Import `[ALL]` `[RUST]` `[TS]`

**Description:** Import tokens from other authenticator apps.

**Acceptance Criteria:**
- [ ] File picker accepts: .json, .txt, .png, .jpg
- [ ] Auto-detect format: Google Authenticator (protobuf), Aegis (JSON), 2FAS (JSON), Ente Auth (JSON), plain otpauth URI list (text)
- [ ] Show list of parsed tokens with checkboxes (all selected by default)
- [ ] User can deselect tokens they don't want to import
- [ ] Import selected tokens into vault
- [ ] Show count: "Imported N tokens"
- [ ] Show warning: "Your import file contains unencrypted secrets. Consider deleting it."
- [ ] Malformed file → show "Could not parse file" with format-specific error
- [ ] Zero tokens found → show "No tokens found in file"

### F10: Export `[ALL]` `[RUST]`

**Description:** Export tokens for migration or backup.

**Acceptance Criteria:**
- [ ] Two export options: Encrypted (KeyForge format) and Plain text (otpauth URIs)
- [ ] Encrypted export: prompt for export password → derive key → encrypt all tokens → save file
- [ ] Plain text export: show BIG warning "This file will contain your unencrypted secrets. Anyone who obtains this file can access your accounts." → require confirmation → generate file
- [ ] Export file is saved via system file picker (user chooses location)
- [ ] `[EXTENSION]` Export triggers browser download
- [ ] Encrypted export format is documented so other tools could implement import

### F11: Copy to Clipboard `[ALL]`

**Description:** Copy TOTP/HOTP codes to clipboard.

**Acceptance Criteria:**
- [ ] Tap/click on code card → code copied to clipboard (without space — "123456" not "123 456")
- [ ] Visual feedback: card flashes, Toast "Copied"
- [ ] Auto-clear clipboard after configurable timeout (default 30s)
- [ ] Auto-clear only if clipboard still contains the code KeyForge copied (don't clear user's other content)
- [ ] On code refresh, if the old code was copied, clear it from clipboard immediately
- [ ] `[DESKTOP]` Enter key copies selected card's code
- [ ] `[DESKTOP]` Cmd/Ctrl+1 through Cmd/Ctrl+9 copies code at that position

### F12: Auto-Lock `[ALL]`

**Description:** Automatically lock the vault after a period of inactivity.

**Acceptance Criteria:**
- [ ] Configurable timeout: 30s, 60s, 2min, 5min, 15min, Never. Default: 60s.
- [ ] Timer resets on any user interaction (tap, click, key press, scroll)
- [ ] When timer expires: zeroize key from memory, show lock screen
- [ ] `[DESKTOP]` Lock on window minimize (configurable, default: On)
- [ ] `[DESKTOP]` Lock on system screen lock (if detectable)
- [ ] `[MOBILE]` Lock when app goes to background (configurable, default: On)
- [ ] `[EXTENSION]` Lock managed by chrome.alarms (popup close doesn't reset state)
- [ ] Manual lock: `[DESKTOP]` Cmd/Ctrl+L, tray menu "Lock". `[MOBILE]` lock button in toolbar. `[EXTENSION]` lock button.

### F13: Search and Filter `[ALL]` `[UI]`

**Description:** Filter the token list by issuer or account name.

**Acceptance Criteria:**
- [ ] Search input at top of Home screen
- [ ] Filter is instant (as-you-type, no debounce needed for local data)
- [ ] Matches on issuer name OR account name (case-insensitive)
- [ ] Non-matching cards visually dim (opacity 0.3) or are hidden (configurable)
- [ ] Clear button (X) restores full list
- [ ] `[DESKTOP]` Cmd/Ctrl+F focuses search input
- [ ] Empty search result → show "No tokens match your search"

### F14: Reorder Tokens `[ALL]` `[UI]`

**Description:** User-defined ordering of tokens.

**Acceptance Criteria:**
- [ ] `[DESKTOP]` Drag and drop cards to reorder
- [ ] `[MOBILE]` Long press → enter reorder mode → drag handles appear → drag to reorder
- [ ] `[EXTENSION]` Drag and drop with drag handle
- [ ] New order persists to vault (sort_order column updated)
- [ ] New order survives lock/unlock and app restart

### F15: Edit Token `[ALL]` `[UI]`

**Description:** Edit a token's display metadata.

**Acceptance Criteria:**
- [ ] Editable fields: issuer, account name
- [ ] Non-editable fields: algorithm, digits, period, type, secret (changing these would break the token)
- [ ] Changes persist to vault immediately
- [ ] Navigate to detail screen via long-press context menu or dedicated button

### F16: Delete Token `[ALL]` `[UI]`

**Description:** Remove a token from the vault.

**Acceptance Criteria:**
- [ ] Delete requires confirmation dialog
- [ ] Dialog text: "Delete [Issuer] token? You will lose 2FA access for this service. This cannot be undone."
- [ ] Confirm → token removed from vault → card animates out → Toast "Token deleted"
- [ ] Cancel → no action
- [ ] Deletion is permanent (no undo in Phase 1 — undo is a Phase 4 consideration with backup)

### F17: Settings `[ALL]` `[UI]`

**Description:** User-configurable settings.

**Acceptance Criteria:**
- [ ] All settings from UI-SPEC.md Settings Screen are implemented
- [ ] Settings persist across lock/unlock and app restart
- [ ] Change master password requires current password verification
- [ ] Delete all data requires triple confirmation with typed confirmation
- [ ] `[EXTENSION]` Settings accessible from options page (full page, not popup)

### F18: System Tray `[DESKTOP]`

**Description:** Desktop system tray presence.

**Acceptance Criteria:**
- [ ] Tray icon always visible when app is running
- [ ] Icon is monochrome, adapts to OS theme (light icon on dark menubar and vice versa)
- [ ] Left click: toggle window visibility
- [ ] Right click: context menu with Show/Hide, Lock Vault, Quit
- [ ] Close button minimizes to tray (does not quit). Quit is only via tray menu or Cmd/Ctrl+Q.
- [ ] `[Linux]` Works on both X11 and Wayland

### F19: Deep Linking `[DESKTOP]` `[MOBILE]` `[NATIVE]`

**Description:** Register as handler for otpauth:// URIs.

**Acceptance Criteria:**
- [ ] App registers for `otpauth://` URI scheme on install
- [ ] Clicking an `otpauth://` link opens KeyForge (or brings to foreground)
- [ ] If vault is locked: show lock screen first → after unlock → show token confirmation
- [ ] If vault is unlocked: show token confirmation directly
- [ ] `[macOS]` Registered in Info.plist
- [ ] `[Windows]` Registered in Windows Registry
- [ ] `[Linux]` Registered in .desktop file
- [ ] `[Android]` Intent filter for `otpauth://` scheme
- [ ] `[iOS]` URL scheme registered in Info.plist

### F20: Landing Page `[WEB]`

**Description:** Marketing/landing page for the project.

**Acceptance Criteria:**
- [ ] See LANDING-PAGE.md for full spec
- [ ] Lives in `sites/landing/`
- [ ] Static site (no server required)
- [ ] Deployed separately from the app

---

## Milestones

### Milestone 1: Foundation

- [ ] Monorepo scaffolded (pnpm workspace, turborepo, Cargo workspace)
- [ ] Rust crates created with empty modules and test scaffolding
- [ ] `keyforge-crypto` fully implemented and tested (TOTP, HOTP, KDF, AEAD)
- [ ] `keyforge-vault` fully implemented and tested (SQLCipher, CRUD, migrations)
- [ ] TypeScript packages scaffolded (core, shared, ui)
- [ ] TOTP/HOTP TypeScript implementation (for extension) implemented and tested
- [ ] otpauth URI parser implemented and tested
- [ ] Design system tokens defined (CSS custom properties)
- [ ] CI pipeline: lint + test + coverage

### Milestone 2: Desktop App

- [ ] Tauri desktop app scaffolded
- [ ] Tauri commands wired to Rust crates
- [ ] All UI components built
- [ ] Vault creation and unlock flow working
- [ ] Token CRUD (add, edit, delete, reorder) working
- [ ] QR scan from image/screen working
- [ ] Clipboard copy with auto-clear working
- [ ] Auto-lock working
- [ ] Settings working
- [ ] System tray working
- [ ] Deep linking working
- [ ] Keyboard shortcuts working
- [ ] E2E tests for desktop passing

### Milestone 3: Mobile App

- [ ] Tauri mobile app scaffolded
- [ ] Android and iOS builds compile
- [ ] All shared UI renders correctly on mobile viewports
- [ ] Camera QR scanning working (both platforms)
- [ ] Biometric unlock working (both platforms)
- [ ] Platform keychain integration working (both platforms)
- [ ] Mobile-specific UX adjustments (larger tap targets, haptics, gestures)
- [ ] E2E tests for mobile passing (at least emulator)

### Milestone 4: Chrome Extension

- [ ] Extension scaffolded with Manifest V3
- [ ] Popup renders shared UI components
- [ ] TypeScript TOTP/HOTP working (Web Crypto API)
- [ ] chrome.storage vault adapter working
- [ ] QR scan from tab working
- [ ] Auto-lock via chrome.alarms working
- [ ] Options page with settings working
- [ ] Extension is < 500 KB zipped
- [ ] E2E tests for extension passing

### Milestone 5: Import/Export + Polish

- [ ] Import from all specified formats working
- [ ] Export (encrypted and plain) working
- [ ] All animations and micro-interactions polished
- [ ] Accessibility audit passed
- [ ] Performance audit (vault unlock < 1s, code generation < 1ms)
- [ ] All E2E tests passing on all platforms
- [ ] Coverage thresholds met

### Milestone 6: Landing Page + Release

- [ ] Landing page built and deployed
- [ ] Desktop installers built and signed (macOS notarized, Windows signed)
- [ ] Android APK/AAB built and signed
- [ ] iOS IPA built and signed
- [ ] Extension zip built
- [ ] GitHub Release created
- [ ] Store submissions (Chrome Web Store, Google Play, App Store)
