import { useState } from "react";
import { Plus } from "lucide-react";
import { parseDuration } from "../lib/time";
import { useTimerStore } from "../store/timerStore";

const PRESETS: { label: string; ms: number }[] = [
	{ label: "30s", ms: 30_000 },
	{ label: "1m", ms: 60_000 },
	{ label: "5m", ms: 300_000 },
	{ label: "10m", ms: 600_000 },
	{ label: "15m", ms: 900_000 },
	{ label: "25m", ms: 1_500_000 },
	{ label: "45m", ms: 2_700_000 },
	{ label: "1h", ms: 3_600_000 },
];

export function TimerForm() {
	const [label, setLabel] = useState("");
	const [time, setTime] = useState("5:00");
	const [err, setErr] = useState<string | null>(null);
	const add = useTimerStore((s) => s.add);

	function submit(durationMs: number) {
		if (durationMs <= 0) {
			setErr("Enter a positive duration");
			return;
		}
		add({ label, durationMs });
		setLabel("");
		setTime("5:00");
		setErr(null);
	}

	function onSubmit(e: React.FormEvent) {
		e.preventDefault();
		const ms = parseDuration(time);
		if (ms == null) {
			setErr("Use formats like 90, 5:00, or 1:30:00");
			return;
		}
		submit(ms);
	}

	return (
		<form onSubmit={onSubmit} className="surface rounded-2xl p-3 sm:p-4">
			<div className="flex flex-col gap-2 sm:flex-row sm:items-center">
				<input
					type="text"
					value={label}
					onChange={(e) => setLabel(e.target.value)}
					placeholder="Label (e.g. Eggs)"
					maxLength={32}
					className="border-border bg-surface-2 text-text placeholder:text-text-subtle focus:border-accent flex-1 rounded-xl border px-3 py-2.5 text-sm focus:outline-none"
				/>
				<input
					type="text"
					value={time}
					onChange={(e) => {
						setTime(e.target.value);
						setErr(null);
					}}
					inputMode="numeric"
					placeholder="5:00"
					aria-label="Duration"
					className="border-border bg-surface-2 text-text placeholder:text-text-subtle focus:border-accent w-full rounded-xl border px-3 py-2.5 font-mono text-base focus:outline-none sm:w-28"
				/>
				<button
					type="submit"
					className="btn-press eq-btn inline-flex items-center justify-center gap-1.5 rounded-xl px-4 py-2.5 text-sm"
				>
					<Plus size={16} />
					Add
				</button>
			</div>
			{err && <p className="text-danger mt-2 text-xs">{err}</p>}
			<div className="mt-3 flex flex-wrap gap-1.5">
				{PRESETS.map((p) => (
					<button
						type="button"
						key={p.label}
						onClick={() => {
							setTime(p.label);
							setErr(null);
							submit(p.ms);
						}}
						className="btn-press chip rounded-lg px-2.5 py-1 text-xs"
					>
						{p.label}
					</button>
				))}
			</div>
		</form>
	);
}
