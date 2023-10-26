import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";


export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  css: {
    preprocessorOptions: {
      sass: {
        additionalData: `@import "src/style/mixins"\n`
      }
    }
  }
}));
