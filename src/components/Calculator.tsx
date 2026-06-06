import { useCallback, useEffect, useMemo, useState } from "react";
import { Delete, History as HistoryIcon, X } from "lucide-react";
import { evaluate, formatResult, prettyExpr } from "../lib/calc";
import { useCalcStore } from "../store/calcStore";

const BUTTONS: { label: string; kind: string; value: string; span?: number }[] = [
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

function lastNumber(expr: string): string {
	const m = expr.match(/([0-9.]+)$/);
	return m ? m[0]! : "";
}

function endsWithOperator(expr: string): boolean {
	return /[+\-×÷*/%]$/.test(expr);
}

function safeEvalPreview(expr: string): string | null {
	try {
		return formatResult(evaluate(expr));
	} catch {
		return null;
	}
}

export function Calculator() {
	const [expr, setExpr] = useState("");
	const [result, setResult] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [showHistory, setShowHistory] = useState(false);
	const history = useCalcStore((s) => s.history);
	const pushHistory = useCalcStore((s) => s.push);
	const removeHistory = useCalcStore((s) => s.remove);
	const clearHistory = useCalcStore((s) => s.clear);

	const preview = useMemo(() => (expr ? safeEvalPreview(expr) : null), [expr]);

	const handle = useCallback(
		(kind: string, value: string) => {
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
					const negated = formatResult(-num);
					setExpr(before + negated);
					return;
				}
				if (endsWithOperator(expr)) {
					setExpr(expr + "-");
				}
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
					const r = formatResult(evaluate(expr));
					setResult(r);
					pushHistory({ expression: expr, result: r });
					setExpr(r);
				} catch (e) {
					setError(e instanceof Error ? e.message : "Error");
				}
			}
		},
		[expr, pushHistory],
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
				handle(kind, k);
				return;
			}
			if (k === "Enter" || k === "=") {
				ev.preventDefault();
				handle("eq", "=");
				return;
			}
			if (k === "Backspace") {
				ev.preventDefault();
				backspace();
				return;
			}
			if (k === "Escape" || k.toLowerCase() === "c") {
				ev.preventDefault();
				handle("fn", "AC");
			}
		}
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [handle, backspace, showHistory]);

	return (
		<div className="flex h-full w-full flex-col gap-3 p-3 sm:p-4">
			<div className="surface flex flex-col gap-1 rounded-2xl px-4 py-3">
				<div className="flex items-center justify-between text-xs text-[var(--color-text-muted)]">
					<button
						onClick={() => setShowHistory((s) => !s)}
						className="btn-press inline-flex items-center gap-1.5 rounded-lg px-2 py-1 hover:bg-[var(--color-bg-soft)]"
						aria-label="Toggle history"
					>
						<HistoryIcon size={14} />
						<span>{history.length}</span>
					</button>
					<span aria-live="polite" className="text-[var(--color-danger)]">
						{error ? error : ""}
					</span>
				</div>
				<div
					className="min-h-[2.25rem] overflow-x-auto text-right font-mono text-sm whitespace-nowrap text-[var(--color-text-muted)]"
					style={{ scrollbarWidth: "thin" }}
				>
					{expr ? prettyExpr(expr) : "0"}
				</div>
				<div className="flex items-center justify-between gap-2">
					<div
						className={`truncate text-right font-mono text-4xl font-medium tracking-tight text-[var(--color-text)] sm:text-5xl ${
							error ? "text-[var(--color-danger)]" : ""
						}`}
					>
						{error ? "Error" : (preview ?? result ?? "0")}
					</div>
					<button
						onClick={backspace}
						disabled={!expr}
						className="btn-press flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-2)] text-[var(--color-text-muted)] hover:bg-[var(--color-bg-soft)] disabled:opacity-30"
						aria-label="Backspace"
					>
						<Delete size={18} />
					</button>
				</div>
			</div>

			<div className="grid flex-1 grid-cols-4 gap-2 sm:gap-2.5">
				{BUTTONS.map((b) => {
					const isOp = b.kind === "op";
					const isFn = b.kind === "fn";
					const isEq = b.kind === "eq";
					const base =
						"btn-press flex select-none items-center justify-center rounded-2xl text-xl font-medium sm:text-2xl";
					const cls = `${base} ${
						isEq ? "eq-btn" : isOp ? "op-btn" : isFn ? "fn-btn" : "num-btn"
					} ${b.span === 2 ? "col-span-2 aspect-201/96" : "aspect-square"} sm:aspect-auto sm:min-h-[64px]`;
					return (
						<button
							key={b.label}
							onClick={() => handle(b.kind, b.value)}
							className={cls}
							aria-label={b.label}
						>
							{b.label}
						</button>
					);
				})}
			</div>

			{showHistory && (
				<div className="surface fixed inset-0 z-30 flex flex-col rounded-none p-4 sm:inset-3 sm:rounded-2xl">
					<div className="mb-2 flex items-center justify-between">
						<h2 className="text-lg font-medium text-[var(--color-text)]">History</h2>
						<div className="flex gap-2">
							{history.length > 0 && (
								<button
									onClick={clearHistory}
									className="btn-press rounded-lg px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:bg-[var(--color-bg-soft)]"
								>
									Clear all
								</button>
							)}
							<button
								onClick={() => setShowHistory(false)}
								className="btn-press flex h-8 w-8 items-center justify-center rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-2)] hover:bg-[var(--color-bg-soft)]"
								aria-label="Close history"
							>
								<X size={16} />
							</button>
						</div>
					</div>
					<div className="flex-1 overflow-y-auto rounded-xl">
						{history.length === 0 ? (
							<p className="py-12 text-center text-sm text-[var(--color-text-subtle)]">
								No history yet
							</p>
						) : (
							<ul className="divide-y divide-[var(--color-border)]">
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
											<div className="truncate font-mono text-xs text-[var(--color-text-muted)]">
												{prettyExpr(h.expression)}
											</div>
											<div className="truncate font-mono text-base text-[var(--color-text)]">
												= {h.result}
											</div>
										</button>
										<button
											onClick={() => removeHistory(h.id)}
											className="btn-press opacity-0 transition-opacity group-hover:opacity-100"
											aria-label="Remove entry"
										>
											<X
												size={14}
												className="text-[var(--color-text-subtle)]"
											/>
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
