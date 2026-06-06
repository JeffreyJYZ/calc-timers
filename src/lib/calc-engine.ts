import {
	default as init,
	evaluate as wasmEvaluate,
	try_evaluate as wasmTryEvaluate,
	format_result as wasmFormatResult,
} from "../../crates/calc-engine/pkg/calc_engine";
import type { AngleMode } from "../store/calcStore";

let initPromise: Promise<unknown> | null = null;

async function ensureInit(): Promise<void> {
	if (!initPromise) {
		initPromise = init();
	}
	await initPromise;
}

export async function evaluate(input: string, mode: AngleMode): Promise<string> {
	await ensureInit();
	const result = wasmEvaluate(input, mode);
	return wasmFormatResult(result);
}

export async function tryEvaluate(input: string, mode: AngleMode): Promise<string> {
	await ensureInit();
	const result = wasmTryEvaluate(input, mode);
	if (result === null || result === undefined) return "";
	return wasmFormatResult(result);
}

export async function evaluatePreview(input: string, mode: AngleMode): Promise<string> {
	return await tryEvaluate(input, mode);
}
