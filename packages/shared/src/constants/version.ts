/**
 * Application version, sourced at build time.
 *
 * In production this is overridden by the bundler (e.g. Vite's
 * `define` option) to inject the real version from package.json.
 * During development / tests it defaults to the workspace version.
 */
export const APP_VERSION: string =
  (typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.1.0');

// Ensure the global augmentation compiles without errors.
declare global {
  // eslint-disable-next-line no-var
  var __APP_VERSION__: string | undefined;
}
