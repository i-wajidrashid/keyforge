/**
 * KeyForge desktop application — frontend entry point.
 *
 * This is loaded by the Tauri webview.  It bootstraps the UI by
 * importing shared components from `@keyforge/ui` and wiring up the
 * Tauri command bridge for vault / OTP operations.
 */

const app = document.getElementById("app")!;

app.innerHTML = `
  <main class="lock-screen">
    <h1 class="logo">KeyForge</h1>
    <p class="tagline">Your keys, your devices.</p>
  </main>
`;
