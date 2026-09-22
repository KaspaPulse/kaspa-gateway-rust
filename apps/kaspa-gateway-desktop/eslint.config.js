import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";

export default defineConfig([
  globalIgnores(["node_modules/", "src-tauri/target/", "frontend/vendor/"]),
  {
    name: "kaspa-gateway/frontend",
    files: ["frontend/**/*.js"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
      },
    },
    linterOptions: {
      reportUnusedDisableDirectives: "error",
      reportUnusedInlineConfigs: "error",
    },
    rules: {
      ...js.configs.recommended.rules,
      "no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    name: "kaspa-gateway/settings-contract-control-character-validation",
    files: ["frontend/src/settings-contract.js"],
    rules: {
      "no-control-regex": "off",
    },
  },
]);
