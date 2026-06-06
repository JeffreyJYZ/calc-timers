import { useEffect, useState } from "react";
import { Timer as TimerIcon, Trash2 } from "lucide-react";
import { useTimerStore } from "../store/timerStore";
import { TimerCard } from "./TimerCard";
import { TimerForm } from "./TimerForm";

export function TimerList() {
  const timers = useTimerStore((s) => s.timers);
  const clearFinished = useTimerStore((s) => s.clearFinished);
  const hydrate = useTimerStore((s) => s.hydrateElapsed);
  const tick = useTimerStore((s) => s.tick);
  const [now, setNow] = useState<number>(() => Date.now());

  useEffect(() => {
    hydrate();
  }, [hydrate]);

  useEffect(() => {
    const id = setInterval(() => {
      const t = Date.now();
      setNow(t);
      tick(t);
    }, 250);
    return () => clearInterval(id);
  }, [tick]);

  const hasFinished = timers.some((t) => t.status === "finished");
  const sorted = [...timers].sort((a, b) => {
    const aRem =
      a.status === "running" && a.startedAt ? a.remainingMs - (now - a.startedAt) : a.remainingMs;
    const bRem =
      b.status === "running" && b.startedAt ? b.remainingMs - (now - b.startedAt) : b.remainingMs;
    if (a.status === "finished" && b.status !== "finished") return -1;
    if (b.status === "finished" && a.status !== "finished") return 1;
    return aRem - bRem;
  });

  return (
    <div className="flex h-full w-full flex-col gap-3 p-3 sm:p-4">
      <TimerForm />

      <div className="flex items-center justify-between px-1">
        <h2 className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-muted)]">
          <TimerIcon size={14} />
          {timers.length} {timers.length === 1 ? "timer" : "timers"}
        </h2>
        {hasFinished && (
          <button
            onClick={clearFinished}
            className="btn-press inline-flex items-center gap-1 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] hover:bg-[var(--color-bg-soft)]"
          >
            <Trash2 size={12} /> Clear finished
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto pb-2">
        {sorted.length === 0 ? (
          <div className="surface flex flex-col items-center justify-center gap-2 rounded-2xl px-4 py-12 text-center">
            <TimerIcon size={28} className="text-[var(--color-text-subtle)]" />
            <p className="text-sm text-[var(--color-text-muted)]">No timers yet</p>
            <p className="text-xs text-[var(--color-text-subtle)]">
              Add one above or pick a preset
            </p>
          </div>
        ) : (
          <ul className="flex flex-col gap-2 sm:gap-3">
            {sorted.map((t) => (
              <li key={t.id}>
                <TimerCard timer={t} now={now} />
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
