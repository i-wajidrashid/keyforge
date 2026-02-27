# Landing Page Spec

Marketing site for KeyForge. Lives in `sites/landing/`. Static site. No server.

---

## Purpose

Convince visitors that KeyForge is worth downloading. Communicate the core value proposition in under 10 seconds. Provide download links. Link to source code. Build trust through transparency.

---

## Design Principles

1. **Same aesthetic as the app** — Dark, minimal, monochrome. No AI gradients. No blue. No hero illustrations with floating shapes. No stock photos.
2. **Fast** — Static HTML/CSS with minimal JS. Target < 100 KB total page weight (excluding screenshots). Lighthouse score: 100/100 performance.
3. **One page** — Single page with sections. No multi-page site. No blog (yet). Anchor links for navigation.
4. **No tracking** — Zero analytics scripts. No cookies. No third-party requests. If analytics are needed later, use server-side analytics (Plausible self-hosted or similar). Phase 1: nothing.
5. **Mobile-responsive** — Works on phones. Single-column layout on small screens. No horizontal scroll.

---

## Sections

### 1. Hero

**Content:**

```
KeyForge

Your keys. Your devices. Your rules.

Free, open-source 2FA authenticator.
Local-first. Zero tracking. Zero cloud.

[Download]  [View Source]
```

**Design:**
- Full viewport height
- KeyForge logo centered (monochrome, minimal — a stylized key or lock glyph, no color)
- Tagline below logo
- Two buttons: "Download" (scrolls to download section) and "View Source" (links to GitHub repo)
- Background: `--surface-base` (#0A0A0A). Text: `--text-primary` (#EDEDED).
- No hero image. No animation. Just the words.

### 2. What It Does

**Content:**

```
One app. Every device.

TOTP & HOTP codes for all your accounts.
Desktop, mobile, browser — one codebase, one vault.
```

Followed by a platform icon row:

```
[Windows] [macOS] [Linux] [Android] [iOS] [Chrome]
```

Icons are monochrome glyphs. Not colored brand logos.

**Below:** A single screenshot of the app (dark mode, Home screen with a few tokens visible). The screenshot is the ONLY image on the page. It should be a real screenshot, not a mockup.

### 3. Why KeyForge

Three feature blocks, side by side (stacked on mobile):

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Local       │  │   Open       │  │   Minimal    │
│   First       │  │   Source     │  │              │
│               │  │              │  │              │
│ Your secrets  │  │ Every line   │  │ No bloat.    │
│ never leave   │  │ of code is   │  │ No accounts. │
│ your device.  │  │ auditable.   │  │ No telemetry.│
│ No server.    │  │ No trust     │  │ Just codes.  │
│ No cloud.     │  │ required.    │  │              │
│ No sync       │  │              │  │              │
│ (unless you   │  │              │  │              │
│ choose to).   │  │              │  │              │
└──────────────┘  └──────────────┘  └──────────────┘
```

**Design:**
- Each block has a monochrome icon above the title (a shield, a code bracket, a circle/dot)
- Short, punchy copy. No paragraphs. Every sentence fits on one line.
- No background color difference. Same surface. Separated by spacing.

### 4. Security

**Content:**

```
Built right.

- AES-256 encrypted vault (SQLCipher)
- Argon2id key derivation
- Biometric unlock (Face ID, Touch ID, Windows Hello, fingerprint)
- Zero network calls — no attack surface
- All cryptography in Rust — memory-safe, auditable
- Auto-lock, auto-clear clipboard
```

**Design:**
- Simple bullet list. Monochrome.
- Each bullet has a small check icon (monochrome).
- No colored badges or shields or "security certified" graphics.

### 5. How It Works

Three steps, horizontal (stacked on mobile):

```
1. Create vault          2. Add tokens           3. Use codes
   Set a master             Scan QR or              Tap to copy.
   password.                enter manually.         Done.
```

**Design:**
- Step numbers are large (48px), monochrome.
- Descriptions are one-liners.
- No illustrations. The steps are self-explanatory.

### 6. Coming Soon

**Content:**

```
What's next.

Phase 2 — Web app + optional encrypted cloud backup
Phase 3 — P2P device-to-device sync (no server required)
Phase 4 — Automated backups + recovery

All future features remain local-first and zero-knowledge.
```

**Design:**
- Simple timeline or stacked list.
- Each phase has a short label and one-liner description.
- Muted text color for "coming soon" items.

### 7. Download

**Content:**

```
Get KeyForge.

[Windows]    [macOS]    [Linux]
[Android]    [iOS]      [Chrome Extension]
```

Each platform button links to the appropriate download:
- Windows: GitHub Release (.msi or .exe)
- macOS: GitHub Release (.dmg)
- Linux: GitHub Release (.AppImage) + link to .deb and .rpm
- Android: Google Play Store link
- iOS: App Store link
- Chrome: Chrome Web Store link

Fallback: all downloads available on the GitHub Releases page.

**Design:**
- Platform buttons in a grid (3x2 on desktop, 2x3 or stacked on mobile)
- Each button has platform icon + name
- Monochrome buttons with subtle border. Hover: background lightens slightly.
- Below grid: "Or build from source" link → GitHub repo

### 8. Footer

**Content:**

```
KeyForge — Open source 2FA authenticator.
Source code on GitHub | License: [TBD]

Made by [your name/handle]
```

**Design:**
- Minimal footer. One or two lines.
- Links to GitHub, license.
- No social media icons (unless you have a presence — add later).
- No newsletter signup. No email collection.

---

## Technical Requirements

### Performance

- Total page weight: < 100 KB (excluding app screenshot)
- Screenshot: optimized WebP, < 200 KB, lazy loaded
- No JavaScript frameworks. Vanilla JS for any interactivity (smooth scroll anchor links, that is probably it).
- OR a minimal static site generator (Astro, 11ty) that outputs pure HTML/CSS.
- No client-side rendering. HTML is pre-built.
- Time to First Contentful Paint: < 1 second on 3G
- Lighthouse: 100 performance, 100 accessibility, 100 best practices, 100 SEO

### SEO

- Semantic HTML (`<header>`, `<main>`, `<section>`, `<footer>`)
- `<title>`: "KeyForge — Free Open Source 2FA Authenticator"
- `<meta name="description">`: "Free, open-source, local-first authenticator. TOTP/HOTP 2FA across desktop, mobile, and browser. Zero tracking. Zero cloud."
- Open Graph tags for social sharing (title, description, image — app screenshot)
- `robots.txt` and `sitemap.xml`

### Hosting

- Static hosting: GitHub Pages, Cloudflare Pages, Netlify, or Vercel
- Custom domain (if desired): `keyforge.dev` or similar
- HTTPS (automatic with any of the above hosts)
- No server-side logic. Pure static files.

### Fonts

- Same fonts as the app: Inter (body) and JetBrains Mono (code/monospace accents)
- Self-host fonts (do not use Google Fonts CDN — zero third-party requests)
- Subset fonts to Latin characters only to minimize file size

---

## What NOT to Include

- No comparison table with competitors (comes across as petty for a new project)
- No testimonials or quotes (there are none yet)
- No pricing section (it is free, forever)
- No "trusted by N users" counter (zero users at launch)
- No animated backgrounds, particles, or canvas effects
- No chatbot or support widget
- No cookie banner (there are no cookies)
- No pop-ups of any kind
- No video embeds
- No carousel or slider
- No "built with AI" badge or mention (let the code speak for itself)
