export function formatDuration(ms: number, showHours = true): string {
	const sign = ms < 0 ? "-" : "";
	const total = Math.abs(Math.round(ms / 1000));
	const h = Math.floor(total / 3600);
	const m = Math.floor((total % 3600) / 60);
	const s = total % 60;
	const pad = (n: number) => n.toString().padStart(2, "0");
	if (showHours || h > 0) return `${sign}${pad(h)}:${pad(m)}:${pad(s)}`;
	return `${sign}${pad(m)}:${pad(s)}`;
}

export function parseDuration(input: string): number | null {
	const trimmed = input.trim();
	if (!trimmed) return null;
	if (/^\d+(\.\d+)?$/.test(trimmed)) {
		const sec = Number(trimmed);
		return Number.isFinite(sec) ? Math.round(sec * 1000) : null;
	}
	const parts = trimmed.split(":").map((p) => p.trim());
	if (parts.length > 3) return null;
	const nums = parts.map((p) => Number(p));
	if (nums.some((n) => !Number.isFinite(n) || n < 0)) return null;
	let h = 0;
	let m = 0;
	let s = 0;
	if (parts.length === 3) {
		const [hh, mm, ss] = nums;
		h = hh ?? 0;
		m = mm ?? 0;
		s = ss ?? 0;
	} else if (parts.length === 2) {
		const [mm, ss] = nums;
		m = mm ?? 0;
		s = ss ?? 0;
	} else {
		s = nums[0] ?? 0;
	}
	if (m >= 60 || s >= 60) return null;
	return (h * 3600 + m * 60 + s) * 1000;
}
