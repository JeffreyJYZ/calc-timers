export type TokenType = "num" | "op" | "lparen" | "rparen";

export interface Token {
  type: TokenType;
  value: string;
}

function isDigit(c: string): boolean {
  return c >= "0" && c <= "9";
}

export function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  const s = input.replace(/\s+/g, "");
  let i = 0;
  while (i < s.length) {
    const c = s[i];
    if (c === undefined) break;
    if (isDigit(c) || c === ".") {
      let j = i;
      let dots = 0;
      while (j < s.length) {
        const cj = s[j];
        if (cj === undefined) break;
        if (isDigit(cj)) {
          j++;
        } else if (cj === ".") {
          dots++;
          if (dots > 1) throw new Error("Invalid number");
          j++;
        } else break;
      }
      const value = s.slice(i, j);
      tokens.push({ type: "num", value });
      i = j;
      continue;
    }
    if (c === "+" || c === "-" || c === "*" || c === "/" || c === "%" || c === "(" || c === ")") {
      const type: TokenType = c === "(" ? "lparen" : c === ")" ? "rparen" : "op";
      tokens.push({ type, value: c });
      i++;
      continue;
    }
    throw new Error(`Unexpected character: ${c}`);
  }
  return tokens;
}

const PRECEDENCE: Record<string, number> = { "+": 1, "-": 1, "*": 2, "/": 2, "%": 2 };

function isOpToken(t: Token): boolean {
  return t.type === "op";
}

export function toRPN(tokens: Token[]): Token[] {
  const out: Token[] = [];
  const ops: Token[] = [];
  let prev: Token | null = null;
  for (const t of tokens) {
    if (t.type === "num") {
      out.push(t);
    } else if (t.type === "op") {
      const isUnary = prev === null || prev.type === "op" || prev.type === "lparen";
      if (isUnary && t.value === "-") {
        out.push({ type: "num", value: "0" });
      }
      while (ops.length > 0) {
        const top = ops[ops.length - 1];
        if (
          top &&
          isOpToken(top) &&
          PRECEDENCE[top.value] !== undefined &&
          PRECEDENCE[t.value] !== undefined &&
          PRECEDENCE[top.value] >= PRECEDENCE[t.value]
        ) {
          const popped = ops.pop();
          if (popped) out.push(popped);
        } else break;
      }
      ops.push(t);
    } else if (t.type === "lparen") {
      ops.push(t);
    } else if (t.type === "rparen") {
      let found = false;
      while (ops.length > 0) {
        const top = ops.pop();
        if (!top) break;
        if (top.type === "lparen") {
          found = true;
          break;
        }
        out.push(top);
      }
      if (!found) throw new Error("Mismatched parentheses");
    }
    prev = t;
  }
  while (ops.length > 0) {
    const top = ops.pop();
    if (!top) break;
    if (top.type === "lparen" || top.type === "rparen") throw new Error("Mismatched parentheses");
    out.push(top);
  }
  return out;
}

export function evaluate(input: string): number {
  if (!input.trim()) return 0;
  const tokens = tokenize(input);
  const rpn = toRPN(tokens);
  const stack: number[] = [];
  for (const t of rpn) {
    if (t.type === "num") {
      stack.push(Number(t.value));
    } else if (t.type === "op") {
      const b = stack.pop();
      const a = stack.pop();
      if (a === undefined || b === undefined) throw new Error("Invalid expression");
      switch (t.value) {
        case "+":
          stack.push(a + b);
          break;
        case "-":
          stack.push(a - b);
          break;
        case "*":
          stack.push(a * b);
          break;
        case "/":
          if (b === 0) throw new Error("Division by zero");
          stack.push(a / b);
          break;
        case "%":
          if (b === 0) throw new Error("Division by zero");
          stack.push(a % b);
          break;
        default:
          throw new Error(`Unknown op: ${t.value}`);
      }
    }
  }
  if (stack.length !== 1) throw new Error("Invalid expression");
  const result = stack[0];
  if (result === undefined || !Number.isFinite(result)) throw new Error("Math error");
  return Math.round(result * 1e12) / 1e12;
}

export function formatResult(n: number): string {
  if (!Number.isFinite(n)) return "Error";
  if (Math.abs(n) < 1e-10) return "0";
  const abs = Math.abs(n);
  if (abs >= 1e16 || (abs > 0 && abs < 1e-6)) {
    return n.toExponential(6).replace(/\.?0+e/, "e");
  }
  const fixed = n.toFixed(10);
  return Number(fixed).toString();
}

export function prettyExpr(expr: string): string {
  return expr.replace(/\*/g, "×").replace(/\//g, "÷").replace(/-/g, "−");
}
