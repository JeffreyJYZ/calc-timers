import {
	Pause,
	Play,
	RotateCcw,
	Trash2,
	Bell,
	BellOff,
	Volume2,
	VolumeX,
	Vibrate,
	X,
} from "lucide-react";
import type { Timer } from "../store/timerStore";
import { useTimerStore } from "../store/timerStore";
import { formatDuration } from "../lib/time";

interface Props {
	timer: Timer;
	now: number;
}

export function TimerCard({ timer, now }: Props) {
	const start = useTimerStore((s) => s.start);
	const pause = useTimerStore((s) => s.pause);
	const reset = useTimerStore((s) => s.reset);
	const remove = useTimerStore((s) => s.remove);
	const update = useTimerStore((s) => s.update);

	const elapsed = timer.status === "running" && timer.startedAt ? now - timer.startedAt : 0;
	const remaining = Math.max(0, timer.remainingMs - elapsed);
	const total = timer.totalMs;
	const progress = total > 0 ? Math.min(1, (total - remaining) / total) : 0;
	const isRunning = timer.status === "running";
	const isFinished = timer.status === "finished";

	const r = 58;
	const c = 2 * Math.PI * r;
	const dash = c * (1 - progress);

	return (
		<div
			className={`surface relative overflow-hidden rounded-2xl p-4 transition-colors ${
				isFinished ? "border-success" : ""
			}`}
		>
			<div className="flex items-start gap-3">
				<div className="relative h-32 w-32 shrink-0">
					<svg
						className="-rotate-90"
						viewBox="0 0 140 140"
						width="100%"
						height="100%"
						aria-hidden
					>
						<circle
							cx="70"
							cy="70"
							r={r}
							fill="none"
							stroke="var(--color-border)"
							strokeWidth="6"
						/>
						<circle
							cx="70"
							cy="70"
							r={r}
							fill="none"
							stroke={isFinished ? "var(--color-success)" : timer.color}
							strokeWidth="6"
							strokeLinecap="round"
							strokeDasharray={c}
							strokeDashoffset={dash}
							style={{ transition: "stroke-dashoffset 0.4s ease" }}
						/>
					</svg>
					<div className="absolute inset-0 flex flex-col items-center justify-center">
						<div
							className={`font-mono text-xl font-medium tabular-nums ${
								isFinished ? "text-success" : "text-text"
							}`}
						>
							{formatDuration(remaining, false)}
						</div>
						<div className="text-text-subtle mt-0.5 text-[10px] tracking-wider uppercase">
							{isFinished ? "Done" : isRunning ? "Running" : timer.status}
						</div>
					</div>
				</div>

				<div className="min-w-0 flex-1">
					<input
						type="text"
						value={timer.label}
						onChange={(e) => update(timer.id, { label: e.target.value })}
						maxLength={32}
						className="text-text w-full truncate bg-transparent text-base font-medium focus:outline-none"
						aria-label="Timer label"
					/>
					<div className="text-text-subtle mt-0.5 text-xs">
						of {formatDuration(total, total >= 3_600_000)}
					</div>

					<div className="mt-3 flex flex-wrap gap-1.5">
						{isRunning ? (
							<button
								onClick={() => pause(timer.id)}
								className="btn-press chip inline-flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs"
								aria-label="Pause"
							>
								<Pause size={12} /> Pause
							</button>
						) : (
							<button
								onClick={() => start(timer.id)}
								disabled={isFinished}
								className="btn-press eq-btn inline-flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs disabled:opacity-40"
								aria-label="Start"
							>
								<Play size={12} /> {isFinished ? "Done" : "Start"}
							</button>
						)}
						<button
							onClick={() => reset(timer.id)}
							className="btn-press chip inline-flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs"
							aria-label="Reset"
						>
							<RotateCcw size={12} /> Reset
						</button>
						<button
							onClick={() => remove(timer.id)}
							className="btn-press chip hover:!text-danger inline-flex items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs"
							aria-label="Delete"
						>
							<Trash2 size={12} />
						</button>
					</div>

					<div className="mt-2 flex items-center gap-1">
						<button
							onClick={() => update(timer.id, { sound: !timer.sound })}
							className="btn-press text-text-subtle hover:bg-bg-soft hover:text-text flex h-7 w-7 items-center justify-center rounded-md"
							aria-label={timer.sound ? "Mute sound" : "Enable sound"}
							title={timer.sound ? "Sound on" : "Sound off"}
						>
							{timer.sound ? <Volume2 size={13} /> : <VolumeX size={13} />}
						</button>
						<button
							onClick={() => update(timer.id, { vibrate: !timer.vibrate })}
							className="btn-press text-text-subtle hover:bg-bg-soft hover:text-text flex h-7 w-7 items-center justify-center rounded-md"
							aria-label={timer.vibrate ? "Disable vibration" : "Enable vibration"}
							title={timer.vibrate ? "Vibrate on" : "Vibrate off"}
						>
							<Vibrate
								size={13}
								className={timer.vibrate ? "" : "line-through opacity-40"}
							/>
						</button>
						<button
							onClick={() => update(timer.id, { notify: !timer.notify })}
							className="btn-press text-text-subtle hover:bg-bg-soft hover:text-text flex h-7 w-7 items-center justify-center rounded-md"
							aria-label={
								timer.notify ? "Disable notifications" : "Enable notifications"
							}
							title={timer.notify ? "Notify on" : "Notify off"}
						>
							{timer.notify ? <Bell size={13} /> : <BellOff size={13} />}
						</button>
					</div>
				</div>
			</div>
			{isFinished && (
				<button
					onClick={() => remove(timer.id)}
					className="btn-press border-border bg-surface-2 text-text-muted hover:bg-bg-soft absolute top-2 right-2 flex h-6 w-6 items-center justify-center rounded-md border"
					aria-label="Dismiss"
				>
					<X size={12} />
				</button>
			)}
		</div>
	);
}
