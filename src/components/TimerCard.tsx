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
      className={`glass-ice relative overflow-hidden rounded-2xl p-4 transition-all ${
        isFinished ? "border-[var(--color-success)]/50" : "hover:border-[var(--color-accent)]/30"
      }`}
    >
      <div
        className="absolute inset-x-0 top-0 h-1 transition-all"
        style={{ background: timer.color, opacity: isRunning ? 1 : 0.5 }}
        aria-hidden
      />

      <div className="flex items-start gap-3">
        <div className="relative h-32 w-32 shrink-0">
          <svg className="-rotate-90" viewBox="0 0 140 140" width="100%" height="100%" aria-hidden>
            <circle
              cx="70"
              cy="70"
              r={r}
              fill="none"
              stroke="rgba(125, 211, 252, 0.08)"
              strokeWidth="8"
            />
            <circle
              cx="70"
              cy="70"
              r={r}
              fill="none"
              stroke={isFinished ? "var(--color-success)" : timer.color}
              strokeWidth="8"
              strokeLinecap="round"
              strokeDasharray={c}
              strokeDashoffset={dash}
              style={{
                transition: "stroke-dashoffset 0.4s ease",
                filter: `drop-shadow(0 0 6px ${timer.color}66)`,
              }}
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <div
              className={`font-mono text-xl font-semibold tabular-nums ${
                isFinished ? "text-[var(--color-success)]" : "ice-text"
              }`}
              style={
                isFinished
                  ? undefined
                  : { backgroundImage: "none", WebkitTextFillColor: "#f0f9ff", color: "#f0f9ff" }
              }
            >
              {formatDuration(remaining, false)}
            </div>
            <div className="mt-0.5 text-[10px] tracking-wider text-[var(--color-frost)]/50 uppercase">
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
            className="w-full truncate bg-transparent text-base font-medium text-[var(--color-ice)] focus:outline-none"
            aria-label="Timer label"
          />
          <div className="mt-0.5 text-xs text-[var(--color-frost)]/40">
            of {formatDuration(total, total >= 3_600_000)}
          </div>

          <div className="mt-3 flex flex-wrap gap-1.5">
            {isRunning ? (
              <button
                onClick={() => pause(timer.id)}
                className="btn-press inline-flex items-center gap-1 rounded-lg border border-[var(--color-border)]/40 bg-white/5 px-2.5 py-1.5 text-xs text-[var(--color-frost)] hover:bg-white/10"
                aria-label="Pause"
              >
                <Pause size={12} /> Pause
              </button>
            ) : (
              <button
                onClick={() => start(timer.id)}
                disabled={isFinished}
                className="btn-press ice-glow inline-flex items-center gap-1 rounded-lg bg-gradient-to-b from-[var(--color-accent)] to-[var(--color-accent-2)] px-2.5 py-1.5 text-xs font-semibold text-[#02101e] disabled:opacity-40"
                aria-label="Start"
              >
                <Play size={12} /> {isFinished ? "Done" : "Start"}
              </button>
            )}
            <button
              onClick={() => reset(timer.id)}
              className="btn-press inline-flex items-center gap-1 rounded-lg border border-[var(--color-border)]/30 bg-white/5 px-2.5 py-1.5 text-xs text-[var(--color-frost)] hover:bg-white/10"
              aria-label="Reset"
            >
              <RotateCcw size={12} /> Reset
            </button>
            <button
              onClick={() => remove(timer.id)}
              className="btn-press inline-flex items-center gap-1 rounded-lg border border-[var(--color-border)]/30 bg-white/5 px-2.5 py-1.5 text-xs text-[var(--color-frost)] hover:border-[var(--color-danger)]/40 hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]"
              aria-label="Delete"
            >
              <Trash2 size={12} />
            </button>
          </div>

          <div className="mt-2 flex items-center gap-1">
            <button
              onClick={() => update(timer.id, { sound: !timer.sound })}
              className="btn-press flex h-7 w-7 items-center justify-center rounded-md text-[var(--color-frost)]/50 hover:bg-white/5 hover:text-[var(--color-frost)]"
              aria-label={timer.sound ? "Mute sound" : "Enable sound"}
              title={timer.sound ? "Sound on" : "Sound off"}
            >
              {timer.sound ? <Volume2 size={13} /> : <VolumeX size={13} />}
            </button>
            <button
              onClick={() => update(timer.id, { vibrate: !timer.vibrate })}
              className="btn-press flex h-7 w-7 items-center justify-center rounded-md text-[var(--color-frost)]/50 hover:bg-white/5 hover:text-[var(--color-frost)]"
              aria-label={timer.vibrate ? "Disable vibration" : "Enable vibration"}
              title={timer.vibrate ? "Vibrate on" : "Vibrate off"}
            >
              <Vibrate size={13} className={timer.vibrate ? "" : "line-through opacity-40"} />
            </button>
            <button
              onClick={() => update(timer.id, { notify: !timer.notify })}
              className="btn-press flex h-7 w-7 items-center justify-center rounded-md text-[var(--color-frost)]/50 hover:bg-white/5 hover:text-[var(--color-frost)]"
              aria-label={timer.notify ? "Disable notifications" : "Enable notifications"}
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
          className="btn-press absolute top-2 right-2 flex h-6 w-6 items-center justify-center rounded-md bg-white/5 text-white/60 hover:bg-white/10"
          aria-label="Dismiss"
        >
          <X size={12} />
        </button>
      )}
    </div>
  );
}
