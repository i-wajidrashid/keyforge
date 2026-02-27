# CI/CD

GitHub Actions workflows, runners, matrix strategy, caching, secrets, and release automation for the KeyForge monorepo.

This document fills in every implementation detail that TESTING.md left as prose. An implementer reading this should be able to create every `.github/workflows/*.yml` file without ambiguity.

---

## Workflow Files

```
.github/
├── workflows/
│   ├── pr-check.yml          # Runs on every PR — lint, test, build verify
│   ├── main-merge.yml        # Runs on merge to main — E2E, build artifacts
│   ├── release.yml           # Runs on version tag push — sign, publish, deploy
│   └── security-audit.yml    # Runs on schedule (weekly) — dependency audits
└── actions/
    ├── setup-rust/           # Composite action: install Rust toolchain + cache
    ├── setup-pnpm/           # Composite action: install pnpm + node + cache
    └── setup-tauri/          # Composite action: install Tauri system deps
```

---

## Workflow 1: PR Check (`pr-check.yml`)

**Trigger:** Every pull request targeting `main`. Also runs on push to `main` (catches direct pushes).

```
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
```

### Jobs

#### Job 1: `lint`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code (`actions/checkout@v4`)
2. Setup Rust toolchain (stable, with `clippy` and `rustfmt` components)
3. Rust cache (`Swatinem/rust-cache@v2`)
4. Setup pnpm (`pnpm/action-setup@v4`)
5. Setup Node (`actions/setup-node@v4` with `cache: 'pnpm'`)
6. `pnpm install --frozen-lockfile`
7. `cargo fmt --workspace --check` — Fail if Rust code is not formatted
8. `cargo clippy --workspace -- -D warnings` — Fail on any Rust warning
9. `pnpm lint` — ESLint + Prettier check across all TS packages
10. `pnpm typecheck` — TypeScript type checking across all packages

**Fail-fast:** Yes. If lint fails, skip all other jobs (save compute).

#### Job 2: `test-rust`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup Rust toolchain (stable)
3. Rust cache (`Swatinem/rust-cache@v2`, key includes `Cargo.lock` hash)
4. Install system deps for SQLCipher build: `sudo apt-get install -y libclang-dev`
5. `cargo test --workspace` — Run all Rust unit and integration tests
6. Install `cargo-tarpaulin`
7. `cargo tarpaulin --workspace --out xml --output-dir coverage/` — Generate coverage report
8. Upload coverage XML as artifact (`actions/upload-artifact@v4`)
9. Check coverage thresholds:
   - `keyforge-crypto`: fail if < 100% line coverage
   - `keyforge-vault`: fail if < 95% line coverage
10. Post coverage summary as PR comment (use a coverage reporting action or custom script)

**Note:** `cargo-tarpaulin` only works on Linux. This is fine — Rust tests are platform-independent (the crypto and vault code has no platform-specific branches).

#### Job 3: `test-typescript`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup pnpm + Node (with cache)
3. `pnpm install --frozen-lockfile`
4. `pnpm test` — Run all TypeScript unit tests (Vitest or chosen runner)
5. `pnpm test:coverage` — Generate coverage report
6. Upload coverage as artifact
7. Check coverage thresholds:
   - `packages/core`: fail if < 95%
   - `packages/shared`: fail if < 95%
   - `packages/ui`: fail if < 85%

#### Job 4: `build-verify`

**Runs on:** Matrix (see below)

**Purpose:** Verify the project compiles on all target platforms. Does NOT produce release artifacts (that is the release workflow's job).

**Matrix:**

| Runner | What it builds | Why this runner |
|--------|---------------|-----------------|
| `ubuntu-latest` | Desktop Linux + Extension | Linux build deps available |
| `macos-latest` | Desktop macOS | Xcode + macOS SDK available |
| `windows-latest` | Desktop Windows | MSVC + Windows SDK available |

**Steps (per runner):**

1. Checkout code
2. Setup Rust + cache
3. Setup pnpm + Node + cache
4. Install Tauri system dependencies:
   - **Linux:** `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
   - **macOS:** Xcode command line tools (pre-installed on `macos-latest`)
   - **Windows:** Pre-installed on `windows-latest`
5. `pnpm install --frozen-lockfile`
6. `pnpm build:packages` — Build shared packages (core, shared, ui)
7. `cargo build --workspace` — Verify all Rust crates compile
8. Build desktop app: `pnpm --filter desktop build` (Tauri build, skip signing)
9. **Linux only:** Build extension: `pnpm --filter extension build`

**Note:** Mobile builds are NOT in the PR check. They are slow (Android SDK + Gradle), and the Rust crates being compiled on all 3 desktop OSes is sufficient to catch cross-platform issues. Mobile builds run on merge to main.

#### Job 5: `security-audit`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup Rust
3. `cargo install cargo-audit`
4. `cargo audit` — Fail on known vulnerabilities in Rust deps
5. Setup pnpm + Node
6. `pnpm install --frozen-lockfile`
7. `pnpm audit --audit-level=high` — Fail on high/critical npm vulnerabilities

---

## Workflow 2: Main Merge (`main-merge.yml`)

**Trigger:** Push to `main` branch (i.e., after PR merge).

```
on:
  push:
    branches: [main]
```

This workflow runs everything from `pr-check.yml` PLUS E2E tests and mobile build verification.

### Additional Jobs (on top of PR check jobs)

#### Job 6: `e2e-desktop`

**Runs on:** `ubuntu-latest`

**Needs:** `test-rust`, `test-typescript` (only run E2E if unit tests pass)

**Steps:**

1. Checkout code
2. Setup Rust + pnpm + Node + Tauri system deps (Linux)
3. `pnpm install --frozen-lockfile`
4. `pnpm build:packages`
5. Build desktop app in dev mode for testing
6. Install Playwright: `pnpm exec playwright install --with-deps chromium`
7. Run desktop E2E tests with Xvfb (virtual display for headless Linux):
   `xvfb-run pnpm test:e2e:desktop`
8. Upload test results (screenshots, traces) as artifacts on failure

#### Job 7: `e2e-extension`

**Runs on:** `ubuntu-latest`

**Needs:** `test-rust`, `test-typescript`

**Steps:**

1. Checkout code
2. Setup pnpm + Node
3. `pnpm install --frozen-lockfile`
4. Build extension: `pnpm --filter extension build`
5. Install Playwright: `pnpm exec playwright install --with-deps chromium`
6. Run extension E2E tests with Xvfb:
   `xvfb-run pnpm test:e2e:extension`
   - Playwright launches Chrome with the built extension loaded via `--load-extension` flag
   - Extension tests run in headed mode (Chrome extensions require non-headless)
   - Xvfb provides the virtual display
7. Upload test results as artifacts on failure

**Chrome extension E2E specifics:**

- Playwright config MUST use `chromium` channel with launch args:
  - `--disable-extensions-except=/path/to/built/extension`
  - `--load-extension=/path/to/built/extension`
  - `--headless=new` (new headless mode supports extensions) OR use Xvfb with `headless: false`
- Extension ID is dynamic in dev mode. Tests MUST discover it at runtime via `chrome://extensions` page or `chrome.management` API.

#### Job 8: `build-mobile`

**Runs on:** `ubuntu-latest` (Android), `macos-latest` (iOS)

**Needs:** `test-rust`, `test-typescript`

**Matrix:**

| Runner | Target |
|--------|--------|
| `ubuntu-latest` | Android |
| `macos-latest` | iOS |

**Steps (Android on ubuntu-latest):**

1. Checkout code
2. Setup Java 17 (`actions/setup-java@v4` with `distribution: 'temurin'`)
3. Setup Android SDK (`android-actions/setup-android@v3`)
4. Install Android NDK (version required by Tauri — currently r27 or whatever Tauri v2 requires)
5. Setup Rust with Android targets:
   - `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android`
6. Rust cache
7. Setup pnpm + Node
8. `pnpm install --frozen-lockfile`
9. `pnpm build:packages`
10. Initialize Tauri mobile: `pnpm tauri android init` (if gen/ not committed)
11. Build APK: `pnpm tauri android build` (debug, no signing)
12. Upload APK as artifact

**Steps (iOS on macos-latest):**

1. Checkout code
2. Xcode is pre-installed on `macos-latest`
3. Setup Rust with iOS targets:
   - `rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim`
4. Rust cache
5. Setup pnpm + Node
6. `pnpm install --frozen-lockfile`
7. `pnpm build:packages`
8. Initialize Tauri mobile: `pnpm tauri ios init` (if gen/ not committed)
9. Build iOS (simulator, no signing): `pnpm tauri ios build`
10. Upload build as artifact

**Note:** Mobile E2E tests are NOT run in CI in Phase 1. They require real emulators which are slow and flaky in CI. Mobile E2E is a manual gate before release. Automate in Phase 2 if stable emulator CI runners exist.

---

## Workflow 3: Release (`release.yml`)

**Trigger:** Push of a version tag.

```
on:
  push:
    tags:
      - 'v*'
```

When you push `v1.0.0`, this workflow builds, signs, and publishes everything.

### Jobs

#### Job 9: `release-desktop`

**Runs on:** Matrix across all 3 desktop OSes

**Matrix:**

| Runner | Target | Installer format |
|--------|--------|-----------------|
| `ubuntu-latest` | Linux x64 | .AppImage, .deb |
| `macos-latest` | macOS universal (x64 + arm64) | .dmg |
| `windows-latest` | Windows x64 | .msi, .exe (NSIS) |

**Steps:**

1. Checkout code
2. Setup Rust + pnpm + Node + Tauri system deps
3. `pnpm install --frozen-lockfile`
4. `pnpm build:packages`
5. Build desktop app with signing:
   - Uses `tauri-apps/tauri-action@v0` (official Tauri GitHub Action)
   - OR manual `pnpm tauri build` with env vars for signing
6. **macOS signing:** Environment variables from secrets:
   - `APPLE_CERTIFICATE` — Base64-encoded .p12 certificate
   - `APPLE_CERTIFICATE_PASSWORD` — Certificate password
   - `APPLE_SIGNING_IDENTITY` — Certificate identity string
   - `APPLE_API_ISSUER` — App Store Connect API issuer ID
   - `APPLE_API_KEY` — App Store Connect API key ID
   - `APPLE_API_KEY_PATH` — Path to .p8 key file (written from secret)
   - Tauri handles notarization automatically when these env vars are set
7. **Windows signing:** Environment variables:
   - `AZURE_KEY_VAULT_URI` — Azure Key Vault URI (cloud HSM for code signing)
   - `AZURE_CLIENT_ID` — Azure AD app client ID
   - `AZURE_CLIENT_SECRET` — Azure AD app client secret
   - `AZURE_TENANT_ID` — Azure AD tenant ID
   - `AZURE_CERT_NAME` — Certificate name in Key Vault
   - Use `tauri-apps/tauri-action` which supports Azure Key Vault signing
   - OR use `signtool` with Azure SignTool for manual signing
8. **Linux:** No code signing required (AppImage is self-contained)
9. Upload installers as artifacts

#### Job 10: `release-android`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup Java 17 + Android SDK + NDK
3. Setup Rust with Android targets
4. Setup pnpm + Node
5. `pnpm install --frozen-lockfile && pnpm build:packages`
6. Write signing keystore from secret:
   - `ANDROID_KEYSTORE_BASE64` → decode to `keystore.jks`
   - `ANDROID_KEYSTORE_PASSWORD`
   - `ANDROID_KEY_ALIAS`
   - `ANDROID_KEY_PASSWORD`
7. Configure `keystore.properties` for Gradle
8. Build signed release: `pnpm tauri android build -- --release`
9. Upload signed APK + AAB as artifacts

#### Job 11: `release-ios`

**Runs on:** `macos-latest`

**Steps:**

1. Checkout code
2. Setup Rust with iOS targets
3. Setup pnpm + Node
4. Import signing certificate and provisioning profile:
   - `IOS_CERTIFICATE_BASE64` — Distribution certificate
   - `IOS_CERTIFICATE_PASSWORD`
   - `IOS_PROVISIONING_PROFILE_BASE64`
   - Install to macOS Keychain and provisioning profile directory
5. `pnpm install --frozen-lockfile && pnpm build:packages`
6. Build signed IPA: `pnpm tauri ios build -- --release`
7. Upload IPA as artifact

#### Job 12: `release-extension`

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup pnpm + Node
3. `pnpm install --frozen-lockfile`
4. Build extension: `pnpm --filter extension build`
5. Zip the build output: `cd apps/extension/dist && zip -r ../keyforge-extension.zip .`
6. Upload zip as artifact

#### Job 13: `publish-github-release`

**Runs on:** `ubuntu-latest`

**Needs:** `release-desktop`, `release-android`, `release-ios`, `release-extension`

**Steps:**

1. Download all artifacts from previous jobs (`actions/download-artifact@v4`)
2. Create GitHub Release (`softprops/action-gh-release@v2`):
   - Tag name from `github.ref`
   - Release name: `KeyForge ${{ github.ref_name }}`
   - Body: auto-generated from commits since last tag, or from a `CHANGELOG.md` if it exists
   - Attach all artifacts:
     - `keyforge_x64.AppImage`
     - `keyforge_amd64.deb`
     - `keyforge.dmg`
     - `keyforge_x64-setup.exe`
     - `keyforge_x64_en-US.msi`
     - `keyforge-release.apk`
     - `keyforge-release.aab`
     - `keyforge.ipa`
     - `keyforge-extension.zip`
   - Mark as pre-release if tag contains `-beta`, `-alpha`, or `-rc`
   - Mark as latest release otherwise

#### Job 14: `publish-chrome-web-store`

**Runs on:** `ubuntu-latest`

**Needs:** `release-extension`

**Steps:**

1. Download extension zip artifact
2. Upload to Chrome Web Store (`mnao305/chrome-extension-upload@v5` or equivalent):
   - `CHROME_EXTENSION_ID` — Extension ID in the store
   - `CHROME_CLIENT_ID` — Google API OAuth client ID
   - `CHROME_CLIENT_SECRET` — Google API OAuth client secret
   - `CHROME_REFRESH_TOKEN` — Google API OAuth refresh token
3. Publish (or submit for review, depending on action configuration)

**Note:** Chrome Web Store review can take hours to days. The action uploads and submits. Approval is asynchronous.

#### Job 15: `deploy-landing`

**Runs on:** `ubuntu-latest`

**Needs:** none (can run in parallel with builds)

**Steps:**

1. Checkout code
2. Setup pnpm + Node
3. `pnpm install --frozen-lockfile`
4. Build landing page: `pnpm --filter landing build`
5. Deploy:
   - If using GitHub Pages: `actions/deploy-pages@v4`
   - If using Cloudflare Pages: `cloudflare/wrangler-action@v3`
   - If using Netlify: `nwtgck/actions-netlify@v3`
6. Update download links in landing page to point to the new GitHub Release (if download URLs are dynamic)

---

## Workflow 4: Security Audit (`security-audit.yml`)

**Trigger:** Weekly schedule + manual dispatch.

```
on:
  schedule:
    - cron: '0 9 * * 1'  # Every Monday at 9 AM UTC
  workflow_dispatch:       # Manual trigger
```

**Runs on:** `ubuntu-latest`

**Steps:**

1. Checkout code
2. Setup Rust
3. `cargo install cargo-audit`
4. `cargo audit` — Check Rust dependencies for known vulnerabilities
5. Setup pnpm + Node
6. `pnpm install --frozen-lockfile`
7. `pnpm audit --audit-level=moderate`
8. On failure: create a GitHub Issue automatically with the audit report
   - Use `actions/github-script@v7` to create an issue titled "Security: dependency vulnerability detected"
   - Label: `security`, `automated`

---

## Caching Strategy

### Rust Cache

Use `Swatinem/rust-cache@v2`:

- Caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` directory
- Cache key includes: `Cargo.lock` hash, runner OS, Rust version
- Shared between jobs on the same runner OS
- Saves ~3-5 minutes per job (Rust compilation is the slowest step)

Configuration:

```
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: |
      crates/keyforge-crypto
      crates/keyforge-vault
      apps/desktop/src-tauri
      apps/mobile/src-tauri
    cache-on-failure: true
```

### pnpm Cache

Use `actions/setup-node@v4` with built-in pnpm caching:

```
- uses: pnpm/action-setup@v4
- uses: actions/setup-node@v4
  with:
    node-version: 'lts/*'
    cache: 'pnpm'
```

This caches the pnpm store (`~/.local/share/pnpm/store`). On cache hit, `pnpm install --frozen-lockfile` takes < 5 seconds instead of 30+.

### Turborepo Remote Cache (Optional)

Turborepo can cache build outputs remotely so that CI runs share cached results:

- Use Vercel's free remote cache (requires Vercel account) OR self-host with an S3-compatible bucket
- Enable with `TURBO_TOKEN` and `TURBO_TEAM` environment variables
- This is optional but can cut build times by 50%+ when only a few packages changed

### Playwright Cache

Cache Playwright browsers to avoid re-downloading (~300 MB):

```
- uses: actions/cache@v4
  with:
    path: ~/.cache/ms-playwright
    key: playwright-${{ hashFiles('pnpm-lock.yaml') }}
```

---

## Required GitHub Secrets

Every secret the workflows need. Set these in repo Settings → Secrets and variables → Actions.

### Required for PR Checks (minimal)

None. PR checks don't need any secrets. They only lint, test, and verify builds without signing.

### Required for Release — macOS

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 signing certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 certificate |
| `APPLE_SIGNING_IDENTITY` | Certificate identity (e.g., "Developer ID Application: Your Name (TEAMID)") |
| `APPLE_API_ISSUER` | App Store Connect API issuer ID (UUID) |
| `APPLE_API_KEY` | App Store Connect API key ID |
| `APPLE_API_KEY_CONTENT` | Contents of the .p8 API key file |

### Required for Release — Windows

| Secret | Description |
|--------|-------------|
| `AZURE_KEY_VAULT_URI` | Azure Key Vault URI (e.g., https://yourkeyvault.vault.azure.net/) |
| `AZURE_CLIENT_ID` | Azure AD application client ID |
| `AZURE_CLIENT_SECRET` | Azure AD application client secret |
| `AZURE_TENANT_ID` | Azure AD tenant ID |
| `AZURE_CERT_NAME` | Name of the code signing certificate in Key Vault |

**Alternative for Windows (if not using Azure Key Vault):**

| Secret | Description |
|--------|-------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded .pfx certificate (if you have an exportable cert) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the .pfx |

### Required for Release — Android

| Secret | Description |
|--------|-------------|
| `ANDROID_KEYSTORE_BASE64` | Base64-encoded .jks keystore file |
| `ANDROID_KEYSTORE_PASSWORD` | Keystore password |
| `ANDROID_KEY_ALIAS` | Key alias within the keystore |
| `ANDROID_KEY_PASSWORD` | Key password |

### Required for Release — iOS

| Secret | Description |
|--------|-------------|
| `IOS_CERTIFICATE_BASE64` | Base64-encoded .p12 distribution certificate |
| `IOS_CERTIFICATE_PASSWORD` | Certificate password |
| `IOS_PROVISIONING_PROFILE_BASE64` | Base64-encoded .mobileprovision file |

### Required for Release — Chrome Web Store

| Secret | Description |
|--------|-------------|
| `CHROME_EXTENSION_ID` | Extension ID from Chrome Web Store developer dashboard |
| `CHROME_CLIENT_ID` | Google API OAuth 2.0 client ID |
| `CHROME_CLIENT_SECRET` | Google API OAuth 2.0 client secret |
| `CHROME_REFRESH_TOKEN` | OAuth refresh token (generated via consent flow) |

### Required for Release — Landing Page

| Secret | Description |
|--------|-------------|
| `DEPLOY_TOKEN` | Deployment token for hosting provider (Cloudflare API token, Netlify token, or GitHub Pages token) |

### Optional

| Secret | Description |
|--------|-------------|
| `TURBO_TOKEN` | Turborepo remote cache token (Vercel or self-hosted) |
| `TURBO_TEAM` | Turborepo team slug |
| `CODECOV_TOKEN` | Codecov upload token (if using Codecov for coverage reporting) |

---

## Runner Specs and Costs

### GitHub-Hosted Runners Used

| Runner | OS | CPU | RAM | Free tier mins/month |
|--------|----|-----|-----|---------------------|
| `ubuntu-latest` | Ubuntu 22.04+ | 4 vCPU | 16 GB | 2,000 (free repos) |
| `macos-latest` | macOS 14+ (Sonoma) | 3 vCPU (M1) | 7 GB | 200 (10x cost multiplier) |
| `windows-latest` | Windows Server 2022+ | 4 vCPU | 16 GB | 1,000 (2x cost multiplier) |

### Cost Optimization

macOS runners are expensive (10x Linux). Minimize macOS usage:

1. **PR checks:** Only one macOS job (build-verify for macOS compilation). Lint, Rust tests, TS tests all run on Linux.
2. **Main merge:** macOS only for iOS build verification.
3. **Release:** macOS required for macOS signing/notarization and iOS builds. Cannot avoid.

**Estimated CI time per PR:**

| Job | Runner | Time |
|-----|--------|------|
| lint | ubuntu | ~2 min |
| test-rust | ubuntu | ~4 min |
| test-typescript | ubuntu | ~2 min |
| build-verify (linux) | ubuntu | ~5 min |
| build-verify (macos) | macos | ~6 min |
| build-verify (windows) | windows | ~6 min |
| security-audit | ubuntu | ~1 min |
| **Total wall time** | (parallel) | **~8 min** (longest path is macos build) |

**Estimated CI time per release:**

| Job | Runner | Time |
|-----|--------|------|
| All PR check jobs | mixed | ~8 min |
| E2E desktop | ubuntu | ~5 min |
| E2E extension | ubuntu | ~3 min |
| release-desktop (3 OS) | mixed | ~10 min |
| release-android | ubuntu | ~8 min |
| release-ios | macos | ~10 min |
| release-extension | ubuntu | ~2 min |
| publish + deploy | ubuntu | ~2 min |
| **Total wall time** | (parallel) | **~20 min** |

---

## Concurrency

Prevent duplicate workflow runs:

```
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

This means: if you push two commits quickly to a PR, the first run is cancelled and only the second runs. Saves compute and avoids confusion.

For the release workflow, do NOT cancel in progress (you don't want a half-published release):

```
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false
```

---

## Branch Protection Rules

Set these in repo Settings → Branches → Branch protection rules for `main`:

- [x] Require pull request before merging
- [x] Require status checks to pass: `lint`, `test-rust`, `test-typescript`, `build-verify`
- [x] Require branches to be up to date before merging
- [x] Require conversation resolution before merging
- [ ] Do NOT require signed commits (adds friction, optional for personal project)
- [ ] Do NOT require linear history (squash merge is fine, but don't force it)

---

## Release Process

Step-by-step for cutting a release:

1. Update version numbers (in `Cargo.toml`, `package.json`, `tauri.conf.json`, `manifest.json`)
2. Update `CHANGELOG.md` (if maintained)
3. Commit: `git commit -m "chore: bump version to X.Y.Z"`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push origin main --tags`
6. `release.yml` triggers automatically
7. Wait ~20 minutes for all builds
8. Verify GitHub Release has all artifacts
9. Verify Chrome Web Store submission went through
10. Verify landing page updated
11. Mobile store submissions (Play Store, App Store) may require manual review approval

**Version format:** Semantic versioning (`vMAJOR.MINOR.PATCH`). Pre-releases: `vX.Y.Z-beta.1`, `vX.Y.Z-rc.1`.

**Tag triggers:**
- `v*` triggers release workflow
- `v*-beta*` or `v*-rc*` → GitHub Release marked as pre-release
- `v*` without suffix → GitHub Release marked as latest
