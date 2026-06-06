import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type AngleMode = "rad" | "deg";

export interface CalcEntry {
	id: string;
	expression: string;
	result: string;
	at: number;
}

interface CalcState {
	history: CalcEntry[];
	angleMode: AngleMode;
	push: (entry: Omit<CalcEntry, "id" | "at">) => void;
	clear: () => void;
	remove: (id: string) => void;
	setAngleMode: (mode: AngleMode) => void;
	toggleAngleMode: () => void;
}

const MAX_HISTORY = 50;

export const useCalcStore = create<CalcState>()(
	persist(
		(set, get) => ({
			history: [],
			angleMode: "deg",
			push: (entry) => {
				const e: CalcEntry = {
					id: crypto.randomUUID(),
					at: Date.now(),
					...entry,
				};
				const next = [e, ...get().history].slice(0, MAX_HISTORY);
				set({ history: next });
			},
			clear: () => set({ history: [] }),
			remove: (id) => set({ history: get().history.filter((e) => e.id !== id) }),
			setAngleMode: (mode) => set({ angleMode: mode }),
			toggleAngleMode: () => set({ angleMode: get().angleMode === "deg" ? "rad" : "deg" }),
		}),
		{
			name: "calctimers.calc",
			storage: createJSONStorage(() => localStorage),
			version: 2,
			migrate: (persisted) => {
				const p = (persisted ?? {}) as Partial<CalcState>;
				return {
					history: p.history ?? [],
					angleMode: p.angleMode ?? "deg",
				};
			},
		},
	),
);
