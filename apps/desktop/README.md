# KeyForge Desktop

Tauri v2 desktop app for KeyForge — encrypted TOTP/HOTP authenticator.

## Prerequisites

### All platforms

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9 (`corepack enable && corepack prepare pnpm@9.15.0 --activate`)

### Linux

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### macOS

Xcode Command Line Tools (`xcode-select --install`).

### Windows

[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++".

## Development

```bash
# From the monorepo root
pnpm install
cd apps/desktop
pnpm dev            # Starts Vite dev server + Tauri with hot-reload
```

## Build Release Executables

```bash
# From the monorepo root
pnpm install

# Build the desktop app (frontend + Rust)
cd apps/desktop
pnpm build
```

This runs `vite build` (frontend) then `tauri build` (Rust), producing
platform-specific installers in `apps/desktop/src-tauri/target/release/bundle/`:

| Platform | Output |
|----------|--------|
| **Linux** | `.deb`, `.AppImage` in `bundle/deb/`, `bundle/appimage/` |
| **macOS** | `.app`, `.dmg` in `bundle/macos/`, `bundle/dmg/` |
| **Windows** | `.msi`, `.exe` in `bundle/msi/`, `bundle/nsis/` |

### Build for a specific target

```bash
# Linux .deb only
cd apps/desktop && pnpm tauri build --bundles deb

# macOS .dmg only
cd apps/desktop && pnpm tauri build --bundles dmg

# Windows .msi only
cd apps/desktop && pnpm tauri build --bundles msi
```

### Debug build (faster, unoptimized)

```bash
cd apps/desktop && pnpm tauri build --debug
```

## Testing

```bash
# From monorepo root — run all Rust tests including e2e
cargo test --workspace

# Desktop e2e tests only
cargo test -p keyforge-desktop

# TypeScript tests
pnpm -r test
```

## Architecture

```
apps/desktop/
├── index.html                 # HTML shell
├── src/
│   └── main.ts                # Frontend entry point
├── src-tauri/
│   ├── Cargo.toml             # Rust crate
│   ├── tauri.conf.json        # Window, bundle, security config
│   ├── capabilities/          # Tauri v2 permissions
│   ├── src/
│   │   ├── main.rs            # Binary entry point
│   │   ├── lib.rs             # Tauri builder + command registration
│   │   └── commands.rs        # Tauri commands → Rust SDK bridge
│   └── tests/
│       └── e2e_tests.rs       # E2E tests with real SQLCipher DB
```

### Tauri Commands

| Command | Description |
|---------|-------------|
| `vault_create` | Create encrypted vault with Argon2id-derived keys |
| `vault_unlock` | Unlock vault by re-deriving keys from master password |
| `vault_lock` | Lock vault, zeroize keys from memory |
| `vault_is_locked` | Check vault lock state |
| `vault_exists` | Check if vault file exists on disk |
| `token_list` | List all tokens (cached) |
| `token_add` | Add a new token |
| `token_delete` | Delete a token |
| `token_update` | Update token issuer/account |
| `token_reorder` | Reorder tokens |
| `token_increment_counter` | Increment HOTP counter |
| `otp_generate_totp` | Generate TOTP code for stored token |
| `otp_generate_totp_raw` | Generate TOTP code from raw Base32 secret |
| `otp_generate_hotp` | Generate HOTP code for stored token |
| `vault_import_uris` | Import tokens from otpauth:// URIs |
| `vault_export_uris` | Export tokens as otpauth:// URIs |
| `vault_export_encrypted` | Export encrypted backup |
| `vault_import_encrypted` | Import encrypted backup |
| `platform_info` | Get OS and architecture info |
