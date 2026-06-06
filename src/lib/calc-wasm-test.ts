import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import {
	initSync,
	evaluate as wasmEvaluate,
	try_evaluate as wasmTryEvaluate,
	format_result as wasmFormatResult,
} from "../../crates/calc-engine/pkg/calc_engine";
import type { AngleMode } from "../store/calcStore";

const __dirname = dirname(fileURLToPath(import.meta.url));
const wasmPath = resolve(__dirname, "../../crates/calc-engine/pkg/calc_engine_bg.wasm");

let initialized = false;
function ensureInit(): void {
	if (initialized) return;
	const bytes = readFileSync(wasmPath);
	initSync({ module: bytes });
	initialized = true;
}

export async function evaluateAsync(input: string, mode: AngleMode): Promise<string> {
	ensureInit();
	const result = wasmEvaluate(input, mode);
	return wasmFormatResult(result);
}

export async function tryEvaluateAsync(input: string, mode: AngleMode): Promise<string> {
	ensureInit();
	try {
		const result = wasmTryEvaluate(input, mode);
		if (result === null || result === undefined) return "";
		return wasmFormatResult(result);
	} catch {
		return "";
	}
}
