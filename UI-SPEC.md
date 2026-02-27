# UI Spec

Every screen, every interaction, every animation. The design system for KeyForge.

This document defines what KeyForge looks and feels like. No AI aesthetics. No gratuitous gradients. No blue-purple-gradient buttons. Minimal, intentional, dense.

---

## Design Philosophy

1. **Utility over decoration** — This is a tool. Every pixel earns its place. No ornamental elements.
2. **Dark-first** — Dark mode is the primary theme. Light mode is secondary (Phase 1 ships dark only, light can come later).
3. **No color noise** — The app uses a neutral palette. Color is reserved for ONE thing: the countdown urgency indicator. Everything else is monochrome.
4. **Dense but not cramped** — Users want to see as many codes as possible. Maximize information density. Use spacing to create hierarchy, not large padding.
5. **Platform-native feel** — Respect platform conventions. No custom title bars on desktop (use native). No custom scroll behavior on mobile (use native). No custom selection on extension (use native).
6. **Zero learning curve** — If you have used any authenticator before, you know how to use this one. No tutorials needed. The UI is self-evident.
7. **No AI icons** — No sparkle icons, no robot icons, no gradient blobs, no floating abstract shapes. Icons are geometric, simple, monochrome. Use a coherent icon set (Phosphor, Lucide, or Tabler — pick one, stick with it).
8. **No brand color** — KeyForge does not have a "brand color." The logo is monochrome. The app is monochrome. This is intentional.

---

## Color System

### Dark Theme (Primary — Ship First)

```
Surface / Background:
  --surface-base:       #0A0A0A     (app background)
  --surface-raised:     #141414     (cards, code list items)
  --surface-overlay:    #1E1E1E     (modals, dropdowns, popovers)
  --surface-input:      #1A1A1A     (input fields, search bar)
  --surface-hover:      #1F1F1F     (hover state on interactive surfaces)
  --surface-active:     #252525     (active/pressed state)

Text:
  --text-primary:       #EDEDED     (primary text, code digits)
  --text-secondary:     #888888     (issuer, account, labels)
  --text-tertiary:      #555555     (placeholder text, timestamps)
  --text-disabled:      #333333     (disabled elements)

Border:
  --border-subtle:      #1E1E1E     (card borders, dividers)
  --border-default:     #2A2A2A     (input borders, focused elements)
  --border-strong:      #3A3A3A     (high-emphasis borders)

Accent (ONLY for countdown urgency):
  --accent-safe:        #4ADE80     (green — more than 10 seconds left)
  --accent-warning:     #FBBF24     (amber — 5-10 seconds left)
  --accent-danger:      #EF4444     (red — less than 5 seconds left)

Utility:
  --color-success:      #4ADE80     (toast success)
  --color-error:        #EF4444     (error states)
  --color-destructive:  #DC2626     (delete buttons, destructive actions)
```

### Light Theme (Phase 1 stretch goal — ship later if time permits)

```
Surface / Background:
  --surface-base:       #FAFAFA
  --surface-raised:     #FFFFFF
  --surface-overlay:    #FFFFFF
  --surface-input:      #F5F5F5
  --surface-hover:      #F0F0F0
  --surface-active:     #E8E8E8

Text:
  --text-primary:       #171717
  --text-secondary:     #737373
  --text-tertiary:      #A3A3A3
  --text-disabled:      #D4D4D4

Border:
  --border-subtle:      #F0F0F0
  --border-default:     #E5E5E5
  --border-strong:      #D4D4D4
```

### Why No Brand Color

Brand colors create visual noise in a utility app. Every authenticator app has a colored accent (blue, purple, green). KeyForge's differentiation IS the absence of color. The only color in the UI is functional: the countdown timer changes from green to amber to red as the code expires. This is the single most important piece of information and it gets the only color.

---

## Typography

### Font Stack

```
--font-mono:    "JetBrains Mono", "SF Mono", "Cascadia Mono", "Fira Code", monospace
--font-body:    "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif
```

- **Code digits** use the monospace font. TOTP codes are numbers and MUST be rendered in monospace so digits do not shift when the code changes.
- **Everything else** uses the body font.
- Bundle Inter and JetBrains Mono with the app. Do not rely on system availability.

### Type Scale

```
--text-xs:      11px / 1.4    (timestamps, tertiary labels)
--text-sm:      13px / 1.5    (secondary labels, account names)
--text-base:    15px / 1.5    (body text, settings labels)
--text-lg:      18px / 1.4    (issuer names)
--text-code:    28px / 1.0    (TOTP code digits — desktop/extension)
--text-code-mobile: 32px / 1.0 (TOTP code digits — mobile, larger tap targets)
--text-xl:      22px / 1.3    (screen titles)
```

### Code Display Format

Codes are ALWAYS displayed with a space in the middle for readability:

- 6 digits: `123 456`
- 8 digits: `1234 5678`

The space is visual only, not copied to clipboard.

---

## Spacing System

8px base unit. All spacing is a multiple of 4px.

```
--space-1:    4px
--space-2:    8px
--space-3:    12px
--space-4:    16px
--space-5:    20px
--space-6:    24px
--space-8:    32px
--space-10:   40px
--space-12:   48px
```

---

## Screen Inventory

### 1. Lock Screen

**Purpose:** Gate access to the vault. First thing users see if vault exists.

**Layout:**

```
┌──────────────────────────────────┐
│                                  │
│                                  │
│           [KeyForge Logo]        │
│           monochrome, small      │
│                                  │
│     ┌──────────────────────┐     │
│     │ Master Password      │     │
│     └──────────────────────┘     │
│                                  │
│         [ Unlock ]               │
│                                  │
│    [Biometric icon] if available │
│                                  │
│                                  │
└──────────────────────────────────┘
```

**Behavior:**
- On app launch, if vault exists and is locked, show this screen
- Master password field: auto-focused, type=password, show/hide toggle
- Unlock button: triggers key derivation, opens vault
- If biometric is available and configured: show fingerprint/face icon below. Tapping it triggers biometric auth. On success, retrieve key from keychain, unlock vault.
- `[EXTENSION]` Lock screen fills the popup. No biometric option.
- `[MOBILE]` Biometric prompt auto-triggers on screen open (configurable).
- Wrong password: shake animation on input field. Show "Wrong password" error. Clear input. Do NOT reveal how many attempts remain (no lockout in Phase 1 — considered for future).
- During Argon2id derivation (~500ms): show a subtle spinner on the unlock button. Do not freeze the UI.

### 2. Onboarding Screen (First Launch)

**Purpose:** Set up the vault for the first time. Only shown once.

**Flow:**

```
Step 1: Welcome
  "KeyForge — Your keys, your devices."
  Brief one-liner description. No long paragraphs.
  [Get Started] button

Step 2: Create Master Password
  Password input (with strength indicator)
  Confirm password input
  Strength indicator: bar that fills. Colors: red/amber/green.
  Warning text if password is weak. DOES NOT block creation.
  [Create Vault] button

Step 3: Biometric Setup (if available)
  "Enable [Face ID / Touch ID / Fingerprint] to unlock?"
  [Enable] / [Skip]

Step 4: Done
  "Your vault is ready. Add your first token."
  [Add Token] / [Skip for Now]
```

**Behavior:**
- Password strength indicator SHOULD check: length >= 8, contains mixed case, contains numbers/symbols. Show strength as: Weak / Fair / Strong / Very Strong.
- On vault creation: generate salts, derive keys, create SQLCipher database, run initial migration. Show progress indicator during this (~1-2 seconds on mobile).
- `[EXTENSION]` Same flow but in the popup viewport (smaller). Use vertical scrolling if needed.

### 3. Home Screen (Code List)

**Purpose:** The main screen. Shows all tokens and their current codes.

**Layout:**

```
┌──────────────────────────────────┐
│  [Search]                  [+] [⚙]│
├──────────────────────────────────┤
│  ┌────────────────────────────┐  │
│  │ G  GitHub                  │  │
│  │    user@email.com          │  │
│  │    123 456          ◠ 22s  │  │
│  └────────────────────────────┘  │
│  ┌────────────────────────────┐  │
│  │ A  AWS                     │  │
│  │    root-account            │  │
│  │    789 012          ◠ 22s  │  │
│  └────────────────────────────┘  │
│  ┌────────────────────────────┐  │
│  │ D  Discord                 │  │
│  │    myname#1234             │  │
│  │    345 678          ◠ 22s  │  │
│  └────────────────────────────┘  │
│                                  │
│           [scrollable]           │
│                                  │
└──────────────────────────────────┘
```

**Code Card Component:**

Each token is displayed as a card with:

```
┌──────────────────────────────────┐
│ [Icon]  Issuer                   │
│         account@email.com        │
│                                  │
│   1 2 3   4 5 6         ◠ 22s   │
└──────────────────────────────────┘
```

- **Icon:** First letter of issuer in a circle (neutral gray background, white text). If the issuer matches a known service, show a simple monochrome icon for that service (NOT colored brand logos — just a monochrome glyph). Known service icons are a stretch goal.
- **Issuer:** Primary text. Prominent.
- **Account:** Secondary text. Below issuer. Smaller, muted color.
- **Code:** Large monospace digits. Split with space. This is the focal point.
- **Progress ring:** Small circular indicator. Stroke color transitions from `--accent-safe` → `--accent-warning` → `--accent-danger` as time runs out. The ring depletes clockwise.
- **Time remaining:** Small text next to progress ring showing seconds left (e.g., "22s").
- **Tap/click on card:** Copies code to clipboard. Toast: "Copied". Briefly flash the card background to confirm.
- **Long press / right click:** Context menu: Edit, Delete, Copy URI.

**Search:**
- Top of screen. Input field with search icon.
- Filters tokens by issuer or account name as you type. Instant filter, no debounce needed for local data.
- `[DESKTOP]` Keyboard shortcut: Cmd/Ctrl+F focuses search.
- `[EXTENSION]` Search is always visible (small viewport, every character matters).
- Clear button (X) appears when search has text.

**Toolbar:**
- [+] Add token button (opens Add screen)
- [Settings gear] Opens settings
- `[DESKTOP]` Cmd/Ctrl+N opens Add screen
- `[EXTENSION]` Plus icon in top-right corner of popup

**Empty State:**
- When vault has no tokens, show centered:
  ```
  No tokens yet.
  [Add your first token]
  ```
  Button opens Add screen.

**Code Refresh Animation:**
- When the TOTP period expires and codes regenerate:
  - Old code fades out (opacity 1 → 0, 150ms)
  - New code fades in (opacity 0 → 1, 150ms)
  - Progress ring resets to full, color resets to green
  - No jarring flash. Smooth transition.

**Reordering:**
- `[DESKTOP]` Drag and drop to reorder tokens
- `[MOBILE]` Long press to enter reorder mode, drag handles appear
- `[EXTENSION]` Drag and drop (compact area, handle on left side)
- Reorder persists to vault (sort_order column)

### 4. Add Token Screen

**Purpose:** Add a new token to the vault.

**Methods:**

```
┌──────────────────────────────────┐
│  ← Add Token                     │
│                                  │
│  ┌────────────────────────────┐  │
│  │  📷  Scan QR Code          │  │
│  └────────────────────────────┘  │
│  ┌────────────────────────────┐  │
│  │  ⌨️  Enter Manually        │  │
│  └────────────────────────────┘  │
│  ┌────────────────────────────┐  │
│  │  📋  Paste otpauth:// URI  │  │
│  └────────────────────────────┘  │
│  ┌────────────────────────────┐  │
│  │  📁  Import from File      │  │
│  └────────────────────────────┘  │
│                                  │
└──────────────────────────────────┘
```

**Scan QR Code:**
- `[MOBILE]` Opens camera via Tauri barcode scanner plugin. Full screen camera viewfinder. When QR detected, vibrate, parse URI, auto-navigate to confirmation.
- `[DESKTOP]` Two options: (a) Select screen region to scan (screenshot, decode QR from image), (b) Import image file containing QR code.
- `[EXTENSION]` Captures current tab screenshot, scans for QR codes in the image. If found, parse. If not found, show "No QR code found on this page."

**Enter Manually:**

```
┌──────────────────────────────────┐
│  ← Manual Entry                  │
│                                  │
│  Issuer (service name)           │
│  ┌────────────────────────────┐  │
│  │ GitHub                     │  │
│  └────────────────────────────┘  │
│                                  │
│  Account                         │
│  ┌────────────────────────────┐  │
│  │ user@email.com             │  │
│  └────────────────────────────┘  │
│                                  │
│  Secret Key                      │
│  ┌────────────────────────────┐  │
│  │ JBSWY3DPEHPK3PXP          │  │
│  └────────────────────────────┘  │
│                                  │
│  ▼ Advanced                      │
│    Type:       [TOTP ▾]          │
│    Algorithm:  [SHA1 ▾]          │
│    Digits:     [6 ▾]            │
│    Period:     [30 ▾]           │
│                                  │
│  [ Add Token ]                   │
└──────────────────────────────────┘
```

- Advanced options collapsed by default (most users never touch these)
- Secret key field: show/hide toggle, accepts Base32 input, strips spaces automatically
- Validate secret key is valid Base32 before allowing submission
- On add: encrypt secret, store in vault, navigate to Home screen, show Toast

**Paste otpauth URI:**
- Text input for pasting `otpauth://totp/...` URI
- On paste: auto-parse, show confirmation with parsed fields
- If parsing fails: show error "Invalid otpauth URI"

**Import from File:**
- File picker. Accept: `.json`, `.txt`, `.png`, `.jpg` (QR image)
- Auto-detect format (Google Authenticator, Aegis, 2FAS, plain text)
- Show list of found tokens with checkboxes (select which to import)
- Import selected → add to vault → show count: "Imported 5 tokens"

### 5. Token Detail / Edit Screen

**Purpose:** View and edit a single token.

```
┌──────────────────────────────────┐
│  ← Token Detail                  │
│                                  │
│  [Icon]                          │
│  GitHub                          │
│  user@email.com                  │
│                                  │
│  ┌──────────────────────┐        │
│  │    123 456     ◠ 22s │        │
│  └──────────────────────┘        │
│           [Copy]                 │
│                                  │
│  Issuer       GitHub          ✏️ │
│  Account      user@email.com  ✏️ │
│  Algorithm    SHA1               │
│  Digits       6                  │
│  Period       30s                │
│  Type         TOTP               │
│  Added        2026-01-15         │
│                                  │
│  [Delete Token]                  │
└──────────────────────────────────┘
```

- Issuer and account are editable (tap pencil icon → inline edit)
- Algorithm, digits, period, type are read-only (changing these would break the token)
- Delete button: shows confirmation dialog. "Delete this token? You will lose access to 2FA for this service. This cannot be undone."
- `[MOBILE]` Swipe back to return to Home

### 6. Settings Screen

```
┌──────────────────────────────────┐
│  ← Settings                      │
│                                  │
│  SECURITY                        │
│  ├─ Auto-lock timeout     [60s ▾]│
│  ├─ Lock on minimize      [On]   │
│  ├─ Biometric unlock      [On]   │
│  └─ Change master password  [→]  │
│                                  │
│  DISPLAY                         │
│  ├─ Compact mode           [Off] │
│  ├─ Show account names     [On]  │
│  └─ Code font size     [Normal ▾]│
│                                  │
│  CLIPBOARD                       │
│  └─ Auto-clear after      [30s ▾]│
│                                  │
│  DATA                            │
│  ├─ Export tokens            [→]  │
│  ├─ Import tokens            [→]  │
│  └─ Delete all data         [→]  │
│                                  │
│  ABOUT                           │
│  ├─ Version              1.0.0   │
│  ├─ Source code             [→]  │
│  └─ License                 [→]  │
│                                  │
└──────────────────────────────────┘
```

**Settings details:**

- **Auto-lock timeout:** Options: 30s, 60s, 2min, 5min, 15min, Never. Default: 60s. Timer resets on any user interaction.
- **Lock on minimize:** `[DESKTOP]` Lock vault when app is minimized. `[MOBILE]` Lock vault when app goes to background. `[EXTENSION]` Lock when popup closes. Default: On.
- **Biometric unlock:** Toggle. Only shown if biometrics are available. When enabled, store vault key in keychain. When disabled, remove key from keychain.
- **Change master password:** Opens flow: enter current password → enter new password → confirm new password → re-derive keys → re-encrypt vault.
- **Compact mode:** Reduces card height. Hides account name. Shows only issuer and code. For power users with many tokens.
- **Export tokens:** Options: Encrypted file (KeyForge format), Plain text (otpauth URIs). Encrypted export prompts for an export password. Plain text shows BIG warning.
- **Import tokens:** Opens Import flow (same as Add Token → Import from File).
- **Delete all data:** Triple confirmation. "Are you sure?" → "This will delete ALL tokens. Type DELETE to confirm." → wipe vault.

---

## Interactions and Animations

### Micro-interactions

| Interaction | Animation | Duration |
|-------------|-----------|----------|
| Copy code (tap card) | Card background flashes `--surface-active` | 200ms |
| Copy toast appears | Slide up from bottom, fade in | 200ms in, auto-dismiss 2s, 200ms out |
| Wrong password | Input field shakes horizontally | 300ms, ease-out |
| Code refresh | Old code fade out, new code fade in | 150ms each |
| Progress ring depletion | Smooth linear decrease, color transitions | Continuous |
| Add token success | Card slides into list from top | 250ms, ease-out |
| Delete token | Card slides out to left, list collapses | 200ms, ease-in |
| Search filter | Cards that don't match fade out (opacity 0.3) | 100ms |
| Screen transitions | Slide left/right (mobile), crossfade (desktop) | 200ms |
| Vault unlock | Lock icon to unlock icon morph (optional stretch goal) | 300ms |

### Progress Ring Behavior

The progress ring is the most important visual element. It communicates urgency.

- **Full circle → empty:** The ring starts full and depletes clockwise over the TOTP period (30s default)
- **Color transitions:** Smooth gradient transitions, not sudden jumps
  - 30s → 10s remaining: `--accent-safe` (green)
  - 10s → 5s remaining: transitions to `--accent-warning` (amber)
  - 5s → 0s remaining: transitions to `--accent-danger` (red)
- **Stroke width:** 2.5px (subtle, not chunky)
- **Size:** 20px diameter (desktop/extension), 24px diameter (mobile)
- **Ring resets:** When period expires, ring snaps back to full and green. No animation on reset (instant).

### Keyboard Shortcuts `[DESKTOP]`

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl + F | Focus search |
| Cmd/Ctrl + N | Add new token |
| Cmd/Ctrl + L | Lock vault |
| Cmd/Ctrl + , | Open settings |
| Escape | Clear search / close modal / go back |
| Up/Down arrows | Navigate code list |
| Enter | Copy selected code |
| Cmd/Ctrl + 1-9 | Copy code for token at position 1-9 |

---

## Component Library

### CodeCard

The core component. Displays one token and its current code.

**Props:**
- token: Token object (issuer, account, type, algorithm, digits, period, etc.)
- code: string (the current generated code)
- timeLeft: number (seconds remaining)
- compact: boolean (compact mode)
- onCopy: callback
- onEdit: callback
- onDelete: callback

**States:**
- Default: shows code, progress ring
- Copied: brief flash, shows checkmark instead of code for 500ms, then reverts
- Expired: code refreshes (fade animation)
- Compact: reduced height, no account name, smaller code text

### ProgressRing

SVG-based circular progress indicator.

**Props:**
- progress: number (0 to 1, where 1 is full)
- size: number (px)
- strokeWidth: number (px)
- colorSafe: string
- colorWarning: string
- colorDanger: string

**Behavior:** Renders an SVG circle with `stroke-dashoffset` animated based on progress. Color interpolates between the three accent colors based on progress value.

### VaultLock

Lock screen component.

**Props:**
- onUnlock: callback(password: string)
- onBiometric: callback
- biometricAvailable: boolean
- biometricType: "face" | "fingerprint" | "none"
- isLoading: boolean (during key derivation)
- error: string | null

### Toast

Non-blocking notification.

**Props:**
- message: string
- type: "success" | "error" | "info"
- duration: number (ms, default 2000)
- onDismiss: callback

**Behavior:** Positioned bottom-center. Stacks upward if multiple toasts. Auto-dismisses. Swipe to dismiss on mobile.

### Modal

Confirmation dialog.

**Props:**
- title: string
- body: string
- confirmLabel: string
- cancelLabel: string
- destructive: boolean (colors confirm button red)
- onConfirm: callback
- onCancel: callback

**Behavior:** Backdrop overlay (semi-transparent black). Centered. Focus trapped inside modal. Escape key closes.

---

## Responsive Behavior

### Desktop Window

- **Default size:** 380px wide x 580px tall (compact, not a sprawling window)
- **Min size:** 320px x 480px
- **Max size:** 600px x 900px
- **Resizable:** Yes, but the app is designed for narrow widths (it is a utility, not a dashboard)
- **System tray:** App minimizes to system tray. Tray icon shows KeyForge logo. Click to restore. Right-click menu: Lock, Quit.

### Mobile

- Full screen, no window controls
- Bottom navigation is NOT used (the app has so few screens that a back button suffices)
- Swipe back for navigation (iOS native, Android gesture nav)
- Code text is larger (`--text-code-mobile: 32px`)
- Pull-to-refresh is NOT needed (codes auto-refresh)

### Chrome Extension Popup

- **Fixed dimensions:** 360px wide x 520px tall (Chrome extension popup max)
- **No window controls** (Chrome manages the popup lifecycle)
- **Denser layout:** Reduced padding, smaller code text, compact cards by default
- **Popup closes on click outside:** This is browser behavior, not controllable. The extension MUST handle re-open gracefully (re-check lock state, re-render).

---

## Accessibility

- All interactive elements MUST be keyboard-navigable
- Focus indicators MUST be visible (2px outline, offset, using `--border-strong` color)
- Color is NEVER the only indicator (progress ring also shows seconds as text)
- Minimum contrast ratio: 4.5:1 for body text, 3:1 for large text (already satisfied by the color system)
- Screen reader labels on all icons and interactive elements
- `aria-live="polite"` on the code display region (so screen readers announce new codes)
- `role="timer"` on the countdown element
- Reduced motion: respect `prefers-reduced-motion` — disable animations, show static progress bar instead of ring
