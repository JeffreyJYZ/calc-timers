import { useCallback, useEffect, useMemo, useState } from "react";
import { Delete, History as HistoryIcon, X, FunctionSquare } from "lucide-react";
import { evaluate, prettyExpr } from "../lib/calc";
import { useCalcStore, type AngleMode } from "../store/calcStore";

type ButtonKind = "num" | "op" | "fn" | "eq" | "trig" | "const" | "mode" | "drawer";

interface ButtonDef {
	label: string;
	kind: ButtonKind;
	value: string;
	span?: number;
}

const STD_BUTTONS: ButtonDef[] = [
	{ label: "AC", kind: "fn", value: "AC" },
	{ label: "+/−", kind: "fn", value: "SIGN" },
	{ label: "%", kind: "op", value: "%" },
	{ label: "÷", kind: "op", value: "/" },
	{ label: "7", kind: "num", value: "7" },
	{ label: "8", kind: "num", value: "8" },
	{ label: "9", kind: "num", value: "9" },
	{ label: "×", kind: "op", value: "*" },
	{ label: "4", kind: "num", value: "4" },
	{ label: "5", kind: "num", value: "5" },
	{ label: "6", kind: "num", value: "6" },
	{ label: "−", kind: "op", value: "-" },
	{ label: "1", kind: "num", value: "1" },
	{ label: "2", kind: "num", value: "2" },
	{ label: "3", kind: "num", value: "3" },
	{ label: "+", kind: "op", value: "+" },
	{ label: "0", kind: "num", value: "0", span: 2 },
	{ label: ".", kind: "num", value: "." },
	{ label: "=", kind: "eq", value: "=" },
];

const SCI_FUNCS: { label: string; insert: (m: AngleMode) => string }[] = [
	{ label: "sin", insert: () => "sin(" },
	{ label: "cos", insert: () => "cos(" },
	{ label: "tan", insert: () => "tan(" },
	{ label: "asin", insert: () => "asin(" },
	{ label: "acos", insert: () => "acos(" },
	{ label: "atan", insert: () => "atan(" },
	{ label: "sinh", insert: () => "sinh(" },
	{ label: "cosh", insert: () => "cosh(" },
	{ label: "tanh", insert: () => "tanh(" },
	{ label: "log", insert: () => "log(" },
	{ label: "ln", insert: () => "ln(" },
	{ label: "exp", insert: () => "exp(" },
	{ label: "√", insert: () => "sqrt(" },
	{ label: "∛", insert: () => "cbrt(" },
	{ label: "x²", insert: () => "^2" },
	{ label: "xʸ", insert: () => "^" },
	{ label: "|x|", insert: () => "abs(" },
	{ label: "1/x", insert: () => "1/(" },
	{ label: "x!", insert: () => "!" },
	{ label: "mod", insert: () => "mod(" },
	{ label: "nPr", insert: () => "nPr(" },
	{ label: "nCr", insert: () => "nCr(" },
	{ label: "yroot", insert: () => "yroot(" },
	{ label: "rand", insert: () => "random()" },
	{ label: "(", insert: () => "(" },
	{ label: ")", insert: () => ")" },
	{ label: "π", insert: () => "pi" },
	{ label: "e", insert: () => "e" },
];

function lastNumber(expr: string): string {
	const m = expr.match(/([0-9.]+)$/);
	return m ? m[0]! : "";
}

function endsWithOperator(expr: string): boolean {
	return /[+\-×÷*/%^!]$/.test(expr);
}

export function Calculator() {
	const [expr, setExpr] = useState("");
	const [result, setResult] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [showHistory, setShowHistory] = useState(false);
	const [showDrawer, setShowDrawer] = useState(false);
	const [mode, setMode] = useState<"std" | "sci">("std");
	const [second, setSecond] = useState(false);

	const history = useCalcStore((s) => s.history);
	const pushHistory = useCalcStore((s) => s.push);
	const removeHistory = useCalcStore((s) => s.remove);
	const clearHistory = useCalcStore((s) => s.clear);
	const angleMode = useCalcStore((s) => s.angleMode);
	const toggleAngleMode = useCalcStore((s) => s.toggleAngleMode);

	const handle = useCallback(
		async (kind: ButtonKind, value: string) => {
			setError(null);
			if (kind === "fn" && value === "AC") {
				setExpr("");
				setResult("");
				return;
			}
			if (kind === "fn" && value === "SIGN") {
				if (!expr) {
					setExpr("-");
					return;
				}
				const last = lastNumber(expr);
				if (last) {
					const before = expr.slice(0, expr.length - last.length);
					const num = Number(last);
					if (!Number.isFinite(num)) return;
					const negated = String(-num);
					setExpr(before + negated);
					return;
				}
				if (endsWithOperator(expr)) {
					setExpr(expr + "-");
				}
				return;
			}
			if (kind === "fn" && value === "2ND") {
				setSecond((s) => !s);
				return;
			}
			if (kind === "mode" && value === "MODE") {
				setMode((m) => (m === "std" ? "sci" : "std"));
				return;
			}
			if (kind === "drawer" && value === "DRAWER") {
				setShowDrawer((s) => !s);
				return;
			}
			if (kind === "trig") {
				setExpr((e) => e + value);
				return;
			}
			if (kind === "const") {
				setExpr((e) => e + value);
				return;
			}
			if (kind === "op") {
				if (!expr && value === "-") {
					setExpr("-");
					return;
				}
				if (endsWithOperator(expr)) {
					if (value === "-") {
						setExpr(expr + "-");
						return;
					}
					setExpr(expr.slice(0, -1) + value);
					return;
				}
				setExpr(expr + value);
				return;
			}
			if (kind === "num") {
				if (value === ".") {
					const last = lastNumber(expr);
					if (last.includes(".")) return;
					if (!last) {
						setExpr(expr + "0.");
						return;
					}
				}
				setExpr(expr + value);
				return;
			}
			if (kind === "eq") {
				if (!expr) return;
				try {
					const r = await evaluate(expr, angleMode);
					setResult(r);
					pushHistory({ expression: expr, result: r });
					setExpr(r);
				} catch (e) {
					setError(e instanceof Error ? e.message : "Error");
				}
			}
		},
		[expr, pushHistory, angleMode],
	);

	const backspace = useCallback(() => {
		setError(null);
		setExpr((e) => e.slice(0, -1));
	}, []);

	useEffect(() => {
		function onKey(ev: KeyboardEvent) {
			if (showHistory) return;
			const k = ev.key;
			if (/^[0-9.+\-*/%()]$/.test(k)) {
				ev.preventDefault();
				const kind = /[0-9.]/.test(k) ? "num" : "op";
				void handle(kind, k);
				return;
			}
			if (k === "Enter" || k === "=") {
				ev.preventDefault();
				void handle("eq", "=");
				return;
			}
			if (k === "Backspace") {
				ev.preventDefault();
				backspace();
				return;
			}
			if (k === "Escape" || k.toLowerCase() === "c") {
				ev.preventDefault();
				void handle("fn", "AC");
				return;
			}
			if (k === "p" || k === "P") {
				ev.preventDefault();
				setExpr((e) => e + "pi");
				return;
			}
			if (k === "\\") {
				ev.preventDefault();
				toggleAngleMode();
				return;
			}
		}
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [handle, backspace, showHistory, toggleAngleMode]);

	const trigLabel = useCallback((base: string, inv: string) => (second ? inv : base), [second]);
	const trigInsert = useCallback(
		(base: string, inv: string) => (second ? inv : base) + "(",
		[second],
	);

	const sciButtons: ButtonDef[] = useMemo(() => {
		return [
			{ label: "AC", kind: "fn", value: "AC" },
			{ label: second ? "1st" : "2nd", kind: "fn", value: "2ND" },
			{ label: angleMode.toUpperCase(), kind: "fn", value: "ANGLE" },
			{ label: "÷", kind: "op", value: "/" },
			{ label: trigLabel("sin", "asin"), kind: "trig", value: trigInsert("sin", "asin") },

			{ label: "7", kind: "num", value: "7" },
			{ label: "8", kind: "num", value: "8" },
			{ label: "9", kind: "num", value: "9" },
			{ label: "×", kind: "op", value: "*" },
			{ label: trigLabel("cos", "acos"), kind: "trig", value: trigInsert("cos", "acos") },

			{ label: "4", kind: "num", value: "4" },
			{ label: "5", kind: "num", value: "5" },
			{ label: "6", kind: "num", value: "6" },
			{ label: "−", kind: "op", value: "-" },
			{ label: trigLabel("tan", "atan"), kind: "trig", value: trigInsert("tan", "atan") },

			{ label: "1", kind: "num", value: "1" },
			{ label: "2", kind: "num", value: "2" },
			{ label: "3", kind: "num", value: "3" },
			{ label: "+", kind: "op", value: "+" },
			{
				label: second ? "exp" : "ln",
				kind: "trig",
				value: second ? "exp(" : "ln(",
			},

			{ label: second ? "e" : "π", kind: "const", value: second ? "e" : "pi" },
			{ label: "0", kind: "num", value: "0", span: 1 },
			{ label: ".", kind: "num", value: "." },
			{ label: "=", kind: "eq", value: "=" },
			{ label: "√", kind: "trig", value: "sqrt(" },
		];
	}, [second, angleMode, trigLabel, trigInsert]);

	const buttons = mode === "std" ? STD_BUTTONS : sciButtons;

	return (
		<div className="flex h-full w-full flex-col gap-3 p-3 sm:p-4">
			<div className="surface flex flex-col gap-1 rounded-2xl px-4 py-3">
				<div className="text-text-muted flex items-center justify-between text-xs">
					<button
						onClick={() => setShowHistory((s) => !s)}
						className="btn-press hover:bg-bg-soft inline-flex items-center gap-1.5 rounded-lg px-2 py-1"
						aria-label="Toggle history"
					>
						<HistoryIcon size={14} />
						<span>{history.length}</span>
					</button>
					<div className="flex items-center gap-1">
						<button
							onClick={() => handle("mode", "MODE")}
							className={`btn-press hover:bg-bg-soft rounded-lg px-2 py-1 font-mono text-[10px] ${
								mode === "sci" ? "bg-bg-soft" : ""
							}`}
							aria-label="Toggle mode"
						>
							{mode.toUpperCase()}
						</button>
						<button
							onClick={toggleAngleMode}
							className="btn-press hover:bg-bg-soft rounded-lg px-2 py-1 font-mono text-[10px]"
							aria-label="Toggle angle mode"
						>
							{angleMode.toUpperCase()}
						</button>
						<button
							onClick={() => handle("drawer", "DRAWER")}
							className="btn-press hover:bg-bg-soft rounded-lg px-2 py-1"
							aria-label="Open function drawer"
						>
							<FunctionSquare size={14} />
						</button>
					</div>
					<span aria-live="polite" className="text-danger">
						{error ? error : ""}
					</span>
				</div>
				<div
					className="text-text-muted min-h-[2.25rem] overflow-x-auto text-right font-mono text-sm whitespace-nowrap"
					style={{ scrollbarWidth: "thin" }}
				>
					{expr ? prettyExpr(expr) : "0"}
				</div>
				<div className="flex items-center justify-between gap-2">
					<div
						className={`text-text truncate text-right font-mono text-4xl font-medium tracking-tight sm:text-5xl ${
							error ? "text-danger" : ""
						}`}
					>
						{(() => {
							if (error) return "Error";
							if (result && !expr) return result;
							return "0";
						})()}
					</div>
					<button
						onClick={backspace}
						disabled={!expr}
						className="btn-press border-border bg-surface-2 text-text-muted hover:bg-bg-soft flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border disabled:opacity-30"
						aria-label="Backspace"
					>
						<Delete size={18} />
					</button>
				</div>
			</div>

			<div
				className={`grid flex-1 gap-2 sm:gap-2.5 ${
					mode === "std" ? "grid-cols-4" : "grid-cols-5"
				}`}
			>
				{buttons.map((b) => {
					const isOp = b.kind === "op";
					const isFn = b.kind === "fn";
					const isEq = b.kind === "eq";
					const isTrig = b.kind === "trig";
					const isConst = b.kind === "const";
					const base =
						"btn-press flex select-none items-center justify-center rounded-2xl text-xl font-medium sm:text-2xl";
					const cls = `${base} ${
						isEq
							? "eq-btn"
							: isOp
								? "op-btn"
								: isFn
									? "fn-btn"
									: isTrig
										? "trig-btn"
										: isConst
											? "const-btn"
											: "num-btn"
					} ${b.span === 2 ? "col-span-2 aspect-201/96" : "aspect-square"} sm:aspect-auto sm:min-h-[64px]`;
					if (b.kind === "fn" && b.value === "ANGLE") {
						return (
							<button
								key={b.label}
								onClick={toggleAngleMode}
								className={cls}
								aria-label="Toggle angle mode"
							>
								{b.label}
							</button>
						);
					}
					return (
						<button
							key={b.label}
							onClick={() => void handle(b.kind, b.value)}
							className={cls}
							aria-label={b.label}
						>
							{b.label}
						</button>
					);
				})}
			</div>

			{showDrawer && (
				<div className="surface fixed inset-x-0 bottom-0 z-30 max-h-[70vh] overflow-y-auto rounded-t-2xl p-4 sm:inset-x-3 sm:bottom-3 sm:rounded-2xl">
					<div className="mb-3 flex items-center justify-between">
						<h2 className="text-text text-lg font-medium">Functions</h2>
						<button
							onClick={() => setShowDrawer(false)}
							className="btn-press border-border bg-surface-2 hover:bg-bg-soft flex h-8 w-8 items-center justify-center rounded-lg border"
							aria-label="Close drawer"
						>
							<X size={16} />
						</button>
					</div>
					<div className="grid grid-cols-4 gap-2 sm:grid-cols-6">
						{SCI_FUNCS.map((f) => (
							<button
								key={f.label}
								onClick={() => {
									const ins = f.insert(angleMode);
									setExpr((e) => e + ins);
									setShowDrawer(false);
								}}
								className="btn-press bg-surface-2 text-text hover:bg-bg-soft rounded-xl px-2 py-3 text-sm font-medium sm:text-base"
							>
								{f.label}
							</button>
						))}
					</div>
					<div className="text-text-subtle mt-3 text-center text-xs">
						Press <kbd className="bg-bg-soft rounded px-1">\\</kbd> to toggle angle mode
						· <kbd className="bg-bg-soft rounded px-1">p</kbd> for π
					</div>
				</div>
			)}

			{showHistory && (
				<div className="surface fixed inset-0 z-30 flex flex-col rounded-none p-4 sm:inset-3 sm:rounded-2xl">
					<div className="mb-2 flex items-center justify-between">
						<h2 className="text-text text-lg font-medium">History</h2>
						<div className="flex gap-2">
							{history.length > 0 && (
								<button
									onClick={clearHistory}
									className="btn-press text-text-muted hover:bg-bg-soft rounded-lg px-3 py-1.5 text-sm"
								>
									Clear all
								</button>
							)}
							<button
								onClick={() => setShowHistory(false)}
								className="btn-press border-border bg-surface-2 hover:bg-bg-soft flex h-8 w-8 items-center justify-center rounded-lg border"
								aria-label="Close history"
							>
								<X size={16} />
							</button>
						</div>
					</div>
					<div className="flex-1 overflow-y-auto rounded-xl">
						{history.length === 0 ? (
							<p className="text-text-subtle py-12 text-center text-sm">
								No history yet
							</p>
						) : (
							<ul className="divide-border divide-y">
								{history.map((h) => (
									<li
										key={h.id}
										className="group flex items-center justify-between gap-2 py-2"
									>
										<button
											onClick={() => {
												setExpr(h.result);
												setShowHistory(false);
											}}
											className="btn-press flex-1 truncate text-left"
										>
											<div className="text-text-muted truncate font-mono text-xs">
												{prettyExpr(h.expression)}
											</div>
											<div className="text-text truncate font-mono text-base">
												= {h.result}
											</div>
										</button>
										<button
											onClick={() => removeHistory(h.id)}
											className="btn-press opacity-0 transition-opacity group-hover:opacity-100"
											aria-label="Remove entry"
										>
											<X size={14} className="text-text-subtle" />
										</button>
									</li>
								))}
							</ul>
						)}
					</div>
				</div>
			)}
		</div>
	);
}
