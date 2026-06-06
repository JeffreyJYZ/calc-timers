/** @type {import("prettier").Config} */
export default {
	semi: true,
	singleQuote: false,
	trailingComma: "all",
	printWidth: 100,
	tabWidth: 4,
	useTabs: true,
	arrowParens: "always",
	endOfLine: "lf",
	plugins: ["prettier-plugin-tailwindcss"],
};
