import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "./storage";

export interface TimerFinishedPayload {
	id: string;
	label: string;
}

export async function scheduleBackgroundTimer(
	id: string,
	label: string,
	fireAtMs: number,
): Promise<void> {
	if (!isTauri()) return;
	try {
		await invoke("schedule_timer", { id, label, fireAtMs });
	} catch (err) {
		console.warn("schedule_timer failed", err);
	}
}

export async function cancelBackgroundTimer(id: string): Promise<void> {
	if (!isTauri()) return;
	try {
		await invoke("cancel_timer", { id });
	} catch (err) {
		console.warn("cancel_timer failed", err);
	}
}

export async function listScheduledTimers(): Promise<string[]> {
	if (!isTauri()) return [];
	try {
		return await invoke<string[]>("list_scheduled");
	} catch {
		return [];
	}
}

export async function onTimerFinished(
	handler: (payload: TimerFinishedPayload) => void,
): Promise<UnlistenFn> {
	if (!isTauri()) return () => {};
	return listen<TimerFinishedPayload>("timer-finished", (e) => handler(e.payload));
}
