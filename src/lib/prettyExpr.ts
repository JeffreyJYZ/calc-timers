export function prettyExpr(expr: string): string {
	return expr.replace(/\*/g, "×").replace(/\//g, "÷").replace(/-/g, "−");
}
