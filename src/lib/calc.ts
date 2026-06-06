import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./storage";

export function prettyExpr(expr: string): string {
	return expr.replace(/\*/g, "×").replace(/\//g, "÷").replace(/-/g, "−");
}

function tokenize(input: string): string[] {
	const tokens: string[] = [];
	const s = input.replace(/\s+/g, "");
	let i = 0;
	while (i < s.length) {
		const c = s[i];
		if (c === undefined) break;
		if ((c >= "0" && c <= "9") || c === ".") {
			let j = i;
			let dots = 0;
			while (j < s.length) {
				const cj = s[j];
				if (cj === undefined) break;
				if (cj >= "0" && cj <= "9") {
					j++;
				} else if (cj === ".") {
					dots++;
					if (dots > 1) break;
					j++;
				} else break;
			}
			tokens.push(s.slice(i, j));
			i = j;
			continue;
		}
		if ("+-*/%()".includes(c)) {
			tokens.push(c);
			i++;
			continue;
		}
		break;
	}
	return tokens;
}

function toRPN(tokens: string[]): string[] {
	const out: string[] = [];
	const ops: string[] = [];
	const prec: Record<string, number> = { "+": 1, "-": 1, "*": 2, "/": 2, "%": 2 };
	let prev: string | null = null;
	for (const t of tokens) {
		if (t >= "0" || t === "." || /^[0-9]/.test(t)) {
			out.push(t);
		} else if ("+-*/%".includes(t)) {
			const isUnary = prev === null || "+-*/%(".includes(prev);
			if (isUnary && t === "-") out.push("0");
			while (
				ops.length > 0 &&
				ops[ops.length - 1] !== undefined &&
				"+-*/%".includes(ops[ops.length - 1]!) &&
				prec[ops[ops.length - 1]!] !== undefined &&
				prec[t] !== undefined &&
				prec[ops[ops.length - 1]!]! >= prec[t]!
			) {
				out.push(ops.pop()!);
			}
			ops.push(t);
		} else if (t === "(") {
			ops.push(t);
		} else if (t === ")") {
			while (ops.length > 0 && ops[ops.length - 1] !== "(") {
				out.push(ops.pop()!);
			}
			ops.pop();
		}
		prev = t;
	}
	while (ops.length > 0) out.push(ops.pop()!);
	return out;
}

function evalRPN(rpn: string[]): number {
	const st: number[] = [];
	for (const t of rpn) {
		if (!isNaN(Number(t))) {
			st.push(Number(t));
		} else {
			const b = st.pop()!;
			const a = st.pop()!;
			if (t === "+") st.push(a + b);
			else if (t === "-") st.push(a - b);
			else if (t === "*") st.push(a * b);
			else if (t === "/") {
				if (b === 0) throw new Error("Division by zero");
				st.push(a / b);
			} else if (t === "%") {
				if (b === 0) throw new Error("Division by zero");
				st.push(a % b);
			}
		}
	}
	return st[0]!;
}

function formatResult(n: number): string {
	if (!Number.isFinite(n)) throw new Error("Math error");
	if (Math.abs(n) < 1e-10) return "0";
	const abs = Math.abs(n);
	if (abs >= 1e16 || abs < 1e-6) {
		return n.toExponential(6).replace(/\.?0+e/, "e");
	}
	const fixed = n.toFixed(10);
	return Number(fixed).toString();
}

function evaluateSync(input: string): string {
	if (!input.trim()) return "0";
	const tokens = tokenize(input);
	if (tokens.length === 0) return "0";
	const rpn = toRPN(tokens);
	const result = evalRPN(rpn);
	return formatResult(result);
}

export async function evaluate(input: string): Promise<string> {
	if (isTauri()) {
		try {
			return await invoke<string>("eval_expression", { expr: input });
		} catch (err) {
			const msg = typeof err === "string" ? err : (err as Error).message;
			throw new Error(msg, { cause: err });
		}
	}
	return evaluateSync(input);
}

export function evaluatePreview(input: string): string {
	try {
		return evaluateSync(input);
	} catch {
		return "";
	}
}
