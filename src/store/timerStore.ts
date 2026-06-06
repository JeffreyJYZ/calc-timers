import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { chime, vibrate } from "../lib/audio";
import { notify, requestNotificationPermission } from "../lib/notifications";

export type TimerStatus = "idle" | "running" | "paused" | "finished";

export interface Timer {
  id: string;
  label: string;
  totalMs: number;
  remainingMs: number;
  status: TimerStatus;
  startedAt: number | null;
  finishedAt: number | null;
  color: string;
  notify: boolean;
  sound: boolean;
  vibrate: boolean;
}

interface TimerState {
  timers: Timer[];
  add: (input: { label: string; durationMs: number; color?: string }) => void;
  remove: (id: string) => void;
  update: (
    id: string,
    patch: Partial<Pick<Timer, "label" | "color" | "notify" | "sound" | "vibrate">>,
  ) => void;
  start: (id: string) => void;
  pause: (id: string) => void;
  reset: (id: string) => void;
  clearFinished: () => void;
  hydrateElapsed: () => void;
  tick: (now: number) => void;
}

const COLORS = [
  "#38bdf8",
  "#22d3ee",
  "#67e8f9",
  "#a5f3fc",
  "#7dd3fc",
  "#818cf8",
  "#c4b5fd",
  "#34d399",
];

function newId(): string {
  return crypto.randomUUID();
}

export const useTimerStore = create<TimerState>()(
  persist(
    (set, get) => ({
      timers: [],

      add: ({ label, durationMs, color }) => {
        const t: Timer = {
          id: newId(),
          label: label.trim() || "Timer",
          totalMs: durationMs,
          remainingMs: durationMs,
          status: "idle",
          startedAt: null,
          finishedAt: null,
          color: color ?? COLORS[get().timers.length % COLORS.length] ?? "#38bdf8",
          notify: true,
          sound: true,
          vibrate: true,
        };
        set({ timers: [t, ...get().timers] });
      },

      remove: (id) => set({ timers: get().timers.filter((t) => t.id !== id) }),

      update: (id, patch) =>
        set({
          timers: get().timers.map((t) => (t.id === id ? { ...t, ...patch } : t)),
        }),

      start: (id) => {
        requestNotificationPermission();
        set({
          timers: get().timers.map((t) => {
            if (t.id !== id) return t;
            if (t.status === "finished" || t.remainingMs <= 0) return t;
            return {
              ...t,
              status: "running",
              startedAt: Date.now(),
              finishedAt: null,
            };
          }),
        });
      },

      pause: (id) => {
        set({
          timers: get().timers.map((t) => {
            if (t.id !== id || t.status !== "running") return t;
            const elapsed = Date.now() - (t.startedAt ?? Date.now());
            return {
              ...t,
              status: "paused",
              remainingMs: Math.max(0, t.remainingMs - elapsed),
              startedAt: null,
            };
          }),
        });
      },

      reset: (id) =>
        set({
          timers: get().timers.map((t) =>
            t.id === id
              ? {
                  ...t,
                  status: "idle",
                  remainingMs: t.totalMs,
                  startedAt: null,
                  finishedAt: null,
                }
              : t,
          ),
        }),

      clearFinished: () => set({ timers: get().timers.filter((t) => t.status !== "finished") }),

      hydrateElapsed: () => {
        const now = Date.now();
        set({
          timers: get().timers.map((t) => {
            if (t.status !== "running" || t.startedAt == null) return t;
            const elapsed = now - t.startedAt;
            const remaining = t.remainingMs - elapsed;
            if (remaining <= 0) {
              return { ...t, status: "finished", remainingMs: 0, finishedAt: now };
            }
            return { ...t, remainingMs: remaining };
          }),
        });
      },

      tick: (now) => {
        const list = get().timers;
        let changed = false;
        const next = list.map((t) => {
          if (t.status !== "running" || t.startedAt == null) return t;
          const elapsed = now - t.startedAt;
          const remaining = t.remainingMs - elapsed;
          if (remaining <= 0) {
            changed = true;
            if (t.sound) chime();
            if (t.vibrate) vibrate([200, 100, 200, 100, 400]);
            if (t.notify) void notify("Timer finished", t.label);
            return {
              ...t,
              status: "finished" as const,
              remainingMs: 0,
              finishedAt: now,
              startedAt: null,
            };
          }
          return t;
        });
        if (changed) set({ timers: next });
      },
    }),
    {
      name: "calctimers.timers",
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ timers: s.timers }),
    },
  ),
);
