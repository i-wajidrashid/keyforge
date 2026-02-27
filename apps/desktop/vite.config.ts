import { defineConfig } from "vite";

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
    __APP_VERSION__: JSON.stringify(
      (await import("./package.json", { with: { type: "json" } })).default
        .version,
    ),
  },
});
