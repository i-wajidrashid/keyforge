# Platform Spec

What changes per platform. What stays the same. How each shell wraps the shared code.

---

## Platform Matrix

| Capability | Desktop (Win/Mac/Linux) | Android | iOS | Chrome Extension |
|-----------|------------------------|---------|-----|-----------------|
| Vault storage | SQLCipher (local file) | SQLCipher (app private storage) | SQLCipher (app sandbox) | chrome.storage.local (encrypted by app) |
| TOTP/HOTP engine | Rust (keyforge-crypto) | Rust (keyforge-crypto) | Rust (keyforge-crypto) | Pure TypeScript (Web Crypto API) |
| Biometric unlock | macOS: Touch ID. Windows: Windows Hello. Linux: None (password only). | Fingerprint, Face | Face ID, Touch ID | Not available (password only) |
| Keychain | macOS: Keychain Services. Windows: Credential Manager. Linux: libsecret. | Android Keystore | iOS Keychain (Secure Enclave) | chrome.storage.session (in-memory) |
| QR scanning | Screen capture + image decode, OR file import | Camera (Tauri barcode scanner plugin) | Camera (Tauri barcode scanner plugin) | Tab screenshot + image decode |
| Clipboard | Tauri clipboard plugin | Tauri clipboard plugin | Tauri clipboard plugin | navigator.clipboard API |
| Auto-clear clipboard | Rust timer | Rust timer + Android 13+ native | Rust timer | chrome.alarms API |
| Deep linking (otpauth://) | Tauri deep-link plugin (register URI scheme) | Android intent filter | iOS URL scheme / Universal Links | Not applicable |
| System tray | Yes (Tauri tray API) | Not applicable | Not applicable | Not applicable |
| Autostart | Tauri autostart plugin (optional) | Not applicable | Not applicable | Chrome manages lifecycle |
| Notifications | Tauri notification plugin | Tauri notification plugin | Tauri notification plugin | chrome.notifications API |
| Min app size target | < 15 MB installed | < 20 MB APK | < 20 MB IPA | < 500 KB zip |
| Auto-lock trigger | Window minimize, system idle timeout | App backgrounded | App backgrounded | Popup close |

---

## Desktop (Tauri v2)

### Window Configuration

| Property | Value |
|----------|-------|
| Default width | 380px |
| Default height | 580px |
| Min width | 320px |
| Min height | 480px |
| Resizable | Yes |
| Decorations | Native (OS title bar, no custom frame) |
| Transparent | No |
| Always on top | Configurable via settings (default: No) |
| Start minimized | Configurable via settings (default: No) |

### System Tray

- Always present when app is running
- Tray icon: monochrome KeyForge icon (adapts to OS theme — light icon on dark menubar, dark icon on light)
- Left click: show/hide main window
- Right click: context menu
  - Show / Hide
  - Lock Vault
  - Quit

### Deep Linking

Register `keyforge://` and `otpauth://` URI schemes via Tauri deep-link plugin.

- When the user clicks an `otpauth://` link in their browser, KeyForge opens (or comes to foreground) and presents the Add Token confirmation screen with the parsed token.
- `keyforge://` is for internal use (future: sync pairing links).
- On Linux: register `.desktop` file with `MimeType` for the URI scheme.
- On macOS: register in `Info.plist` via Tauri config.
- On Windows: register in Windows Registry via Tauri plugin.

### Keyboard Shortcuts

Implemented via Tauri's global shortcut system or frontend keyboard event listeners.

All shortcuts from UI-SPEC.md apply here. Additionally:

- `Cmd/Ctrl + Q`: Quit app (standard OS shortcut)
- `Cmd/Ctrl + W`: Hide window (not quit — app stays in tray)
- `Cmd/Ctrl + M`: Minimize window

### Auto-Lock Behavior

Lock vault when:

1. Auto-lock timeout expires (configurable, default 60s of no interaction)
2. Window is minimized (if "Lock on minimize" is enabled)
3. System is locked (screen lock detected — platform-dependent)
4. User explicitly locks (Cmd/Ctrl + L or via tray menu)

On lock:
- Zeroize vault key from memory
- Clear any cached codes
- Show lock screen
- If in tray, require unlock on next window show

### Platform-Specific Notes

**macOS:**
- Use `NSApplicationActivationPolicy` to allow the app to run as a menu bar app (no dock icon) if "Start minimized" is enabled
- Touch ID via Tauri biometric plugin (uses LocalAuthentication framework)
- Keychain access via `security` framework bindings in Rust

**Windows:**
- Windows Hello for biometric (fingerprint / face / PIN) via Tauri biometric plugin
- Credential Manager for keychain via Windows Credential API
- Installer: NSIS or WiX (Tauri supports both)

**Linux:**
- No native biometric support in Tauri. Password-only unlock.
- Keychain via `libsecret` (GNOME Keyring / KWallet) — via Tauri keychain plugin or direct Rust bindings
- Distribute as: AppImage (universal), .deb (Debian/Ubuntu), .rpm (Fedora)
- Wayland AND X11 compatibility for system tray (use Tauri's tray implementation which handles both)

---

## Android (Tauri v2 Mobile)

### App Configuration

- Min SDK: API 28 (Android 9) — required for Android Keystore hardware-backed crypto
- Target SDK: Latest stable
- App permissions:
  - `CAMERA` (for QR scanning)
  - `USE_BIOMETRIC` (for fingerprint/face unlock)
  - `RECEIVE_BOOT_COMPLETED` (optional, for autostart)
  - NO internet permission in Phase 1 (this is a strong signal to users that the app makes no network calls)

### QR Scanning

Uses Tauri barcode scanner plugin:
- `scan({ formats: [Format.QR_CODE] })`
- Windowed mode: overlay camera viewfinder on top of webview (less jarring than separate camera activity)
- On successful scan: vibrate (short pulse), parse URI, navigate to confirmation
- Handle permission denied gracefully: show "Camera permission required" with button to open system settings

### Biometric

Uses Tauri biometric plugin:
- Checks availability: `checkStatus()`
- Prompts: `authenticate({ reason: "Unlock KeyForge vault" })`
- Supported types: fingerprint, face, iris
- If no biometric enrolled: fall back to device PIN/pattern (configurable — default: allow device credential fallback)

### Storage

- Vault file stored in app-private internal storage (`/data/data/com.keyforge.app/databases/`)
- NOT on external storage (SD card) — ever
- Android Keystore stores the vault decryption key (hardware-backed on supported devices)
- Keystore key protected with `BiometricPrompt.BIOMETRIC_STRONG` requirement

### Navigation

- Android back button / gesture: navigates back in the app stack
- Back from Home screen: minimize app (do not exit)
- Back from lock screen: minimize app (do not exit)
- Swipe gestures follow Android Material conventions

### Lifecycle

- App goes to background → start auto-lock timer
- App goes to foreground → check if auto-lock timer expired → show lock screen if yes
- Process killed by OS → vault is locked on next cold start (key only in memory)

---

## iOS (Tauri v2 Mobile)

### App Configuration

- Deployment target: iOS 16+ (for modern Keychain and Privacy features)
- Required capabilities:
  - Camera (NSCameraUsageDescription: "KeyForge needs camera access to scan QR codes")
  - Face ID (NSFaceIDUsageDescription: "KeyForge uses Face ID to unlock your vault")
- NO network entitlement in Phase 1

### QR Scanning

Same as Android — Tauri barcode scanner plugin. iOS-specific:
- Camera permission prompt uses the `NSCameraUsageDescription` string
- Haptic feedback on successful scan (`UIImpactFeedbackGenerator`, medium)

### Biometric

Uses Tauri biometric plugin:
- Face ID or Touch ID (depending on device)
- Falls back to device passcode if configured to allow
- iOS Keychain item stored with `kSecAccessControlBiometryCurrentSet` (re-enroll biometric invalidates the key — this is intentional and secure)

### Storage

- Vault file in app sandbox (Documents directory for backup inclusion, or Application Support for no iCloud backup — choose Application Support)
- iOS Keychain for vault key storage
- Keychain item attributes: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` (key not included in iCloud Keychain sync, not available when device is locked)

### Navigation

- iOS swipe-back gesture: supported natively
- No bottom tab bar (app is too simple for tab navigation)
- Modal presentations for Add Token and Settings

### Lifecycle

- App enters background → start auto-lock timer
- App returns to foreground → check timer → lock screen if expired
- App terminated → vault locked on cold start
- `applicationProtectedDataWillBecomeUnavailable` → lock vault immediately (device is about to lock)

### App Store Considerations

- No JIT compilation (Tauri uses WKWebView, which is allowed)
- No downloading remote code
- Privacy nutrition label: No data collected. No data linked to identity. No tracking.

---

## Chrome Extension (Manifest V3)

### Architecture

```
extension/
├── manifest.json           # MV3 manifest
├── service-worker/
│   └── background.ts       # Service worker (auto-lock timer, alarms, message passing)
├── popup/
│   ├── index.html          # Popup entry point
│   ├── popup.ts            # Popup script (mounts UI)
│   └── styles.css          # Popup styles
├── options/
│   ├── index.html          # Options page entry point
│   └── options.ts          # Full settings UI
├── content-scripts/
│   └── qr-detector.ts      # Injected into pages to detect QR codes (optional)
└── _locales/               # Internationalization (stretch goal)
```

### Manifest V3 Configuration

Key manifest fields:

- `manifest_version`: 3
- `permissions`: `["storage", "clipboardWrite", "activeTab", "alarms"]`
- `optional_permissions`: `["tabs"]` (for QR scanning from tab)
- `action`: popup UI
- `background`: service worker
- `options_page`: full settings page
- `content_security_policy`: strict, no remote resources, no eval

### Crypto Strategy

The extension CANNOT run Rust or WebAssembly in the service worker (Manifest V3 restriction). Therefore:

- TOTP/HOTP: Pure TypeScript using Web Crypto API (`SubtleCrypto`)
  - `crypto.subtle.importKey("raw", secret, { name: "HMAC", hash: "SHA-1" }, false, ["sign"])`
  - `crypto.subtle.sign("HMAC", key, counterBuffer)`
  - Dynamic truncation in JS (simple bit operations)
- Key derivation: PBKDF2 via Web Crypto API (Argon2id is NOT available in SubtleCrypto)
  - `crypto.subtle.deriveKey({ name: "PBKDF2", salt, iterations: 600000, hash: "SHA-256" }, ...)`
  - PBKDF2 with high iteration count is the best available option in the browser
  - This is a KNOWN security tradeoff vs Argon2id on Tauri platforms. Document it.
- Encryption: AES-256-GCM via Web Crypto API
  - `crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce }, key, plaintext)`

### Storage

- `chrome.storage.local`: encrypted vault data (tokens, settings). Persists across browser restarts.
- `chrome.storage.session`: derived encryption key. In-memory only. Cleared on browser close. Max 10 MB.
- Storage is NOT accessible to web pages or other extensions.
- The vault format in chrome.storage.local mirrors the SQLCipher schema as JSON:
  ```
  {
    "vault": {
      "version": 1,
      "salt": "base64...",
      "kdf_params": { ... },
      "tokens_encrypted": "base64..."  // AES-256-GCM encrypted JSON array of tokens
    }
  }
  ```

### QR Scanning

1. User clicks "Scan QR" in popup
2. Extension calls `chrome.tabs.captureVisibleTab()` to screenshot the active tab
3. Image is decoded in the popup context using a JS QR decoder library
4. If QR found and contains `otpauth://` → parse and show confirmation
5. If no QR found → show "No QR code found on this page"
6. Screenshot is immediately discarded from memory after scan

### Auto-Lock

- `chrome.alarms` API for auto-lock timer
- On popup open: check if alarm has fired since last interaction
- If alarm fired: vault is locked, show lock screen
- On any user interaction in popup: reset the alarm
- On popup close: alarm continues running in service worker

### Popup Lifecycle

The popup is destroyed and recreated every time it opens/closes. This means:

- State MUST be reconstructable from `chrome.storage.local` and `chrome.storage.session`
- On popup open:
  1. Check `chrome.storage.session` for vault key
  2. If key exists → vault is unlocked → render Home screen
  3. If no key → vault is locked → render Lock screen
- On popup close:
  - Nothing to do (state is in storage, timer is an alarm)

### Size Budget

The extension MUST be small:

- Total zip size: < 500 KB (Chrome Web Store listing quality signal)
- No large dependencies. The UI framework, QR decoder, and crypto are all that's needed.
- Tree-shake aggressively. Dead code elimination in the build.

---

## Cross-Platform Orchestration

### What Is Shared

| Layer | Shared Across |
|-------|--------------|
| `packages/ui` components | ALL platforms |
| `packages/core` business logic | ALL platforms (with runtime-specific adapters) |
| `packages/shared` types/utils | ALL platforms |
| `crates/keyforge-crypto` | Desktop + Mobile (NOT extension) |
| `crates/keyforge-vault` | Desktop + Mobile (NOT extension) |
| Design system tokens (CSS) | ALL platforms |

### What Is Platform-Specific

| Concern | Where It Lives |
|---------|---------------|
| Tauri config | `apps/desktop/tauri.conf.json`, `apps/mobile/tauri.conf.json` |
| Tauri commands (Rust) | `apps/desktop/src-tauri/src/`, `apps/mobile/src-tauri/src/` |
| Android manifest | `apps/mobile/gen/android/` |
| iOS plist | `apps/mobile/gen/apple/` |
| Extension manifest | `apps/extension/manifest.json` |
| Platform capabilities | `apps/*/capabilities/` |
| Storage adapter impl | `packages/core/vault/tauri-adapter.ts`, `extension-adapter.ts` |
| QR scan implementation | Platform-specific in each app shell |

### Desktop vs Mobile Tauri Split

Desktop and mobile are SEPARATE Tauri apps (separate directories under `apps/`) because:

1. Different `tauri.conf.json` (window config vs mobile config)
2. Different capabilities (desktop has tray, autostart; mobile has barcode scanner, biometric)
3. Different generated native projects (mobile has `gen/android/` and `gen/apple/`)
4. Different build commands (`tauri build` vs `tauri android build` / `tauri ios build`)

BUT they share:

1. The same Rust crates (keyforge-crypto, keyforge-vault)
2. The same Tauri commands (copy the command definitions, or use a shared Rust crate for commands)
3. The same frontend code (packages/ui, packages/core)
4. The same design system

The Tauri commands (the `#[tauri::command]` functions) SHOULD live in a shared Rust crate or be duplicated between desktop and mobile src-tauri directories. A shared Rust crate (`crates/keyforge-tauri-commands/`) that both apps depend on is the cleaner approach.
