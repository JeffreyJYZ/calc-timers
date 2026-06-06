import { describe, expect, it } from "vitest";
import { prettyExpr } from "./prettyExpr";
import { evaluateAsync as evaluate, tryEvaluateAsync as tryEvaluate } from "./calc-wasm-test";

const DEG = "deg" as const;
const RAD = "rad" as const;

describe("prettyExpr", () => {
	it("replaces * with ×", () => {
		expect(prettyExpr("2*3")).toBe("2×3");
	});

	it("replaces / with ÷", () => {
		expect(prettyExpr("6/2")).toBe("6÷2");
	});

	it("replaces - with −", () => {
		expect(prettyExpr("5-3")).toBe("5−3");
	});

	it("replaces all operators in mixed expression", () => {
		expect(prettyExpr("2*3+4/2-1")).toBe("2×3+4÷2−1");
	});

	it("leaves other characters alone", () => {
		expect(prettyExpr("sin(pi/2)")).toBe("sin(pi÷2)");
	});
});

describe("evaluate — arithmetic", () => {
	const cases: [string, number][] = [
		["1+1", 2],
		["2*3", 6],
		["10-3", 7],
		["20/4", 5],
		["2+3*4", 14],
		["(2+3)*4", 20],
		["-5+10", 5],
		["2^10", 1024],
		["100/10/2", 5],
		["7%3", 1],
		["-(-5)", 5],
		["-(2+3)", -5],
		["2*(3+4)", 14],
		["0+0", 0],
		["3.14*2", 6.28],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, DEG));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — constants", () => {
	it("pi", async () => {
		expect(Number(await evaluate("pi", DEG))).toBeCloseTo(Math.PI, 9);
	});

	it("e", async () => {
		expect(Number(await evaluate("e", DEG))).toBeCloseTo(Math.E, 9);
	});

	it("2*pi", async () => {
		expect(Number(await evaluate("2*pi", DEG))).toBeCloseTo(2 * Math.PI, 9);
	});
});

describe("evaluate — trig rad", () => {
	const cases: [string, number][] = [
		["sin(0)", 0],
		["cos(0)", 1],
		["tan(0)", 0],
		["sin(pi/2)", 1],
		["cos(pi)", -1],
		["sin(pi)", 0],
		["tan(pi/4)", 1],
	];
	for (const [input, expected] of cases) {
		it(`rad: ${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, RAD));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — trig deg", () => {
	const cases: [string, number][] = [
		["sin(0)", 0],
		["cos(0)", 1],
		["sin(30)", 0.5],
		["cos(60)", 0.5],
		["sin(90)", 1],
		["cos(180)", -1],
		["tan(45)", 1],
	];
	for (const [input, expected] of cases) {
		it(`deg: ${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, DEG));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — inverse trig rad", () => {
	const cases: [string, number][] = [
		["asin(0)", 0],
		["asin(1)", Math.PI / 2],
		["acos(1)", 0],
		["acos(0)", Math.PI / 2],
		["atan(0)", 0],
		["atan(1)", Math.PI / 4],
	];
	for (const [input, expected] of cases) {
		it(`rad: ${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, RAD));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — inverse trig deg", () => {
	const cases: [string, number][] = [
		["asin(0.5)", 30],
		["acos(0.5)", 60],
		["atan(1)", 45],
		["asin(1)", 90],
	];
	for (const [input, expected] of cases) {
		it(`deg: ${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, DEG));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — hyperbolic", () => {
	it("sinh(0)", async () => {
		expect(Number(await evaluate("sinh(0)", DEG))).toBeCloseTo(0, 9);
	});

	it("cosh(0)", async () => {
		expect(Number(await evaluate("cosh(0)", DEG))).toBeCloseTo(1, 9);
	});

	it("tanh(0)", async () => {
		expect(Number(await evaluate("tanh(0)", DEG))).toBeCloseTo(0, 9);
	});
});

describe("evaluate — log/exp", () => {
	const cases: [string, number][] = [
		["log(100)", 2],
		["log(1000)", 3],
		["ln(1)", 0],
		["ln(e)", 1],
		["exp(0)", 1],
		["exp(1)", Math.E],
		["exp(2)", Math.E ** 2],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, DEG));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — roots", () => {
	const cases: [string, number][] = [
		["sqrt(4)", 2],
		["sqrt(9)", 3],
		["sqrt(2)", Math.SQRT2],
		["cbrt(27)", 3],
		["cbrt(-8)", -2],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			const r = Number(await evaluate(input, DEG));
			expect(r).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — rounding", () => {
	const cases: [string, number][] = [
		["abs(-5)", 5],
		["abs(5)", 5],
		["floor(3.7)", 3],
		["ceil(3.2)", 4],
		["round(3.5)", 4],
		["floor(-2.3)", -3],
		["ceil(-2.7)", -2],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			expect(Number(await evaluate(input, DEG))).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — min/max", () => {
	const cases: [string, number][] = [
		["min(1,2)", 1],
		["max(1,2)", 2],
		["min(5,3)", 3],
		["max(-1,-5)", -1],
		["min(7,7)", 7],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			expect(Number(await evaluate(input, DEG))).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — combinatorics", () => {
	it("5!", async () => {
		expect(Number(await evaluate("5!", DEG))).toBe(120);
	});

	it("0!", async () => {
		expect(Number(await evaluate("0!", DEG))).toBe(1);
	});

	const cases: [string, number][] = [
		["nPr(5,2)", 20],
		["nPr(10,3)", 720],
		["nCr(5,2)", 10],
		["nCr(10,3)", 120],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			expect(Number(await evaluate(input, DEG))).toBeCloseTo(expected, 9);
		});
	}
});

describe("evaluate — yroot/pow", () => {
	const cases: [string, number][] = [
		["yroot(8,3)", 2],
		["yroot(16,4)", 2],
		["yroot(27,3)", 3],
	];
	for (const [input, expected] of cases) {
		it(`${input} = ${expected}`, async () => {
			expect(Number(await evaluate(input, DEG))).toBeCloseTo(expected, 9);
		});
	}

	it("pow(2,10)", async () => {
		expect(Number(await evaluate("pow(2,10)", DEG))).toBe(1024);
	});
});

describe("evaluate — error handling", () => {
	it("rejects division by zero", async () => {
		await expect(evaluate("1/0", DEG)).rejects.toThrow();
	});

	it("rejects empty function args", async () => {
		await expect(evaluate("min()", DEG)).rejects.toThrow();
	});

	it("rejects unknown identifier", async () => {
		await expect(evaluate("xyz", DEG)).rejects.toThrow();
	});

	it("rejects mismatched parens", async () => {
		await expect(evaluate("(1+2", DEG)).rejects.toThrow();
	});
});

describe("tryEvaluate", () => {
	it("returns formatted string for valid", async () => {
		expect(await tryEvaluate("1+1", DEG)).toBe("2");
	});

	it("returns empty string for invalid", async () => {
		expect(await tryEvaluate("1/0", DEG)).toBe("");
		expect(await tryEvaluate("xyz", DEG)).toBe("");
		expect(await tryEvaluate("(1+2", DEG)).toBe("");
	});

	it("returns empty for partial trailing op", async () => {
		expect(await tryEvaluate("1+", DEG)).toBe("");
	});
});

describe("formatResult", () => {
	it("rounds very small numbers to 0", async () => {
		expect(await evaluate("1e-20", DEG)).toBe("0");
	});
});
