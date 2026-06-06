import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import tailwindcss from "@tailwindcss/vite";
import fs from "fs";

const host = process.env.TAURI_DEV_HOST;
const pkg = JSON.parse(fs.readFileSync("package.json", "utf-8"));

export default defineConfig(async () => ({
  base: "./",
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (id.includes("node_modules")) {
            if (id.includes("react-dom") || id.includes("react/")) {
              return "vendor-react";
            }
            if (id.includes("lucide") || id.includes("recharts") || id.includes("sonner")) {
              return "vendor-ui";
            }
            if (id.includes("qr-code-styling") || id.includes("html-to-image") || id.includes("qrcode")) {
              return "vendor-qr";
            }
            if (id.includes("dompurify")) {
              return "vendor-util";
            }
          }
        },
      },
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "0.0.0.0",
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
}));