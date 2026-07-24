import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
      // Plain Map/Set used as local scratch values or replaced wholesale on
      // every update (not mutated in place) don't need Svelte's reactive
      // wrappers — reassignment already triggers reactivity.
      "svelte/prefer-svelte-reactivity": "off",
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },
  {
    ignores: ["dist/", "build/", "node_modules/", "src-tauri/"],
  },
);
