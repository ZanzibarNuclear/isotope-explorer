import path from "node:path";
import { fileURLToPath } from "node:url";
import vue from "@vitejs/plugin-vue";
import wasm from "vite-plugin-wasm";
import { defineConfig } from "vite";

const root = fileURLToPath(new URL(".", import.meta.url));
const repoRoot = path.resolve(root, "../..");
const wasmPkg = path.join(repoRoot, "crates/nuclear-sim-wasm/pkg");

export default defineConfig({
  plugins: [vue(), wasm()],
  resolve: {
    alias: {
      "@wasm": wasmPkg,
    },
  },
  server: {
    fs: {
      allow: [repoRoot],
    },
  },
});
