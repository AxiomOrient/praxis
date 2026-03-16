import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;
const isDebug = !!process.env.TAURI_ENV_DEBUG;
const isWindows = process.env.TAURI_ENV_PLATFORM === "windows";

export default defineConfig({
  clearScreen: false,
  plugins: [svelte()],
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  base: isDebug ? "/" : "./",
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: isWindows ? "chrome105" : "safari13",
    minify: isDebug ? false : "esbuild",
    sourcemap: isDebug,
  },
});
