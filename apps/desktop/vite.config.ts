import { defineConfig } from "vite";
import { readFileSync } from "node:fs";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

export default defineConfig({
  // Prevent vite from obscuring Rust build errors
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  // Inject APP_VERSION at build time from package.json
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
});
