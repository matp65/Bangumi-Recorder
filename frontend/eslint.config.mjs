import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTypescript from "eslint-config-next/typescript";

export default defineConfig([
  ...nextVitals,
  ...nextTypescript,
  globalIgnores(["dist/**", "node_modules/**", "src/**/*.vue", "src/main.ts", "src/router/**", "src/stores/**"]),
]);
