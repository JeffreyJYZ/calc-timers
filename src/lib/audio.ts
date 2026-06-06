let ctx: AudioContext | null = null;

function getCtx(): AudioContext | null {
	if (typeof window === "undefined") return null;
	if (!ctx) {
		const AC =
			window.AudioContext ??
			(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
		if (!AC) return null;
		ctx = new AC();
	}
	if (ctx.state === "suspended") void ctx.resume();
	return ctx;
}

export function beep(durationMs = 600, freq = 880, volume = 0.25): void {
	const c = getCtx();
	if (!c) return;
	const osc = c.createOscillator();
	const gain = c.createGain();
	osc.type = "sine";
	osc.frequency.value = freq;
	gain.gain.setValueAtTime(0, c.currentTime);
	gain.gain.linearRampToValueAtTime(volume, c.currentTime + 0.01);
	gain.gain.exponentialRampToValueAtTime(0.0001, c.currentTime + durationMs / 1000);
	osc.connect(gain);
	gain.connect(c.destination);
	osc.start();
	osc.stop(c.currentTime + durationMs / 1000);
}

export function chime(): void {
	beep(180, 880, 0.2);
	setTimeout(() => beep(180, 1175, 0.2), 200);
	setTimeout(() => beep(360, 1568, 0.22), 400);
}

export function vibrate(pattern: number | number[]): void {
	if (typeof navigator !== "undefined" && "vibrate" in navigator) {
		try {
			navigator.vibrate(pattern);
		} catch {
			/* ignore */
		}
	}
}
