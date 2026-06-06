// @ts-check

import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import { defineConfig } from "eslint/config";

export default defineConfig([
	{
		ignores: [
			"dist",
			"src-tauri/target",
			"node_modules",
			"android",
			"ios",
			"*.tsbuildinfo",
			"vite.config.js",
			"scripts",
		],
	},
	js.configs.recommended,
	tseslint.configs.recommended,
	{
		files: ["**/*.{ts,tsx}"],
		languageOptions: {
			ecmaVersion: 2022,
			globals: { ...globals.browser, ...globals.es2022, ...globals.node },
		},
		plugins: {
			// @ts-expect-error ESLint 10 Plugin type rejects react-hooks v5 configs.flat shape; runtime works.
			"react-hooks": reactHooks,
			"react-refresh": { rules: reactRefresh.rules },
		},
		rules: {
			...reactHooks.configs.flat.recommended.rules,
			"react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
			"@typescript-eslint/no-unused-vars": [
				"warn",
				{ argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
			],
			"@typescript-eslint/no-unnecessary-type-assertion": "off",
			"@typescript-eslint/no-explicit-any": "off",
			"no-undef": "off",
			"no-useless-assignment": "off",
		},
	},
	{
		files: ["vite.config.ts", "eslint.config.js", "prettier.config.js"],
		languageOptions: {
			globals: { ...globals.node },
		},
		rules: {
			"@typescript-eslint/no-unused-vars": "off",
		},
	},
]);
