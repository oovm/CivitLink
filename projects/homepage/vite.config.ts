import { resolve } from "node:path";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";
import docsPlugin from "./vite-plugin-docs";

export default defineConfig({
    plugins: [vue(), docsPlugin()],
    resolve: {
        alias: {
            "@": resolve(__dirname, "src"),
        },
    },
    assetsInclude: ["**/*.md", "**/*.wasm"],
    server: {
        fs: {
            allow: ["../.."],
        },
    },
    build: {
        target: "esnext",
    },
    optimizeDeps: {
        exclude: ["shiki"],
    },
});
