import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export interface CalcEntry {
	id: string;
	expression: string;
	result: string;
	at: number;
}

interface CalcState {
	history: CalcEntry[];
	push: (entry: Omit<CalcEntry, "id" | "at">) => void;
	clear: () => void;
	remove: (id: string) => void;
}

const MAX_HISTORY = 50;

export const useCalcStore = create<CalcState>()(
	persist(
		(set, get) => ({
			history: [],
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
		}),
		{
			name: "calctimers.calc",
			storage: createJSONStorage(() => localStorage),
		},
	),
);
