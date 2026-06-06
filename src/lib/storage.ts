import { LazyStore } from "@tauri-apps/plugin-store";

const STORE_FILE = "calctimers.dat";
let store: LazyStore | null = null;

function isTauri(): boolean {
	return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export { isTauri };

function getStore(): LazyStore {
	if (!store) store = new LazyStore(STORE_FILE);
	return store;
}

export async function loadJSON<T>(key: string, fallback: T): Promise<T> {
	try {
		if (isTauri()) {
			const s = getStore();
			const v = await s.get<T>(key);
			return v ?? fallback;
		}
		const raw = localStorage.getItem(key);
		return raw ? (JSON.parse(raw) as T) : fallback;
	} catch (err) {
		console.warn("loadJSON failed", key, err);
		return fallback;
	}
}

export async function saveJSON<T>(key: string, value: T): Promise<void> {
	try {
		if (isTauri()) {
			const s = getStore();
			await s.set(key, value);
			await s.save();
			return;
		}
		localStorage.setItem(key, JSON.stringify(value));
	} catch (err) {
		console.warn("saveJSON failed", key, err);
	}
}
