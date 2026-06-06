#[derive(Debug, Clone, PartialEq)]
enum TokType {
	Num,
	Op,
	LParen,
	RParen,
}

#[derive(Debug, Clone)]
struct Token {
	kind: TokType,
	value: String,
}

fn is_digit(c: char) -> bool {
	c >= '0' && c <= '9'
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
	let s: String = input.split_whitespace().collect();
	let bytes = s.as_bytes();
	let mut tokens = Vec::new();
	let mut i = 0;
	while i < bytes.len() {
		let c = bytes[i] as char;
		if is_digit(c) || c == '.' {
			let mut j = i;
			let mut dots = 0;
			while j < bytes.len() {
				let cj = bytes[j] as char;
				if is_digit(cj) {
					j += 1;
				} else if cj == '.' {
					dots += 1;
					if dots > 1 {
						return Err("Invalid number".into());
					}
					j += 1;
				} else {
					break;
				}
			}
			if j == i {
				return Err("Invalid number".into());
			}
			tokens.push(Token {
				kind: TokType::Num,
				value: s[i..j].to_string(),
			});
			i = j;
			continue;
		}
		match c {
			'+' | '-' | '*' | '/' | '%' | '(' | ')' => {
				let kind = if c == '(' {
					TokType::LParen
				} else if c == ')' {
					TokType::RParen
				} else {
					TokType::Op
				};
				tokens.push(Token {
					kind,
					value: c.to_string(),
				});
				i += 1;
			}
			_ => return Err(format!("Unexpected character: {}", c)),
		}
	}
	Ok(tokens)
}

fn precedence(op: &str) -> i32 {
	match op {
		"+" | "-" => 1,
		"*" | "/" | "%" => 2,
		_ => 0,
	}
}

fn to_rpn(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
	let mut out: Vec<Token> = Vec::new();
	let mut ops: Vec<Token> = Vec::new();
	let mut prev: Option<&Token> = None;
	for t in tokens.iter() {
		match t.kind {
			TokType::Num => out.push(t.clone()),
			TokType::Op => {
				let is_unary = match prev {
					None => true,
					Some(p) => p.kind == TokType::Op || p.kind == TokType::LParen,
				};
				if is_unary && t.value == "-" {
					out.push(Token {
						kind: TokType::Num,
						value: "0".into(),
					});
				}
				while let Some(top) = ops.last() {
					if top.kind == TokType::Op
						&& precedence(&top.value) != 0
						&& precedence(&t.value) != 0
						&& precedence(&top.value) >= precedence(&t.value)
					{
						out.push(ops.pop().unwrap());
					} else {
						break;
					}
				}
				ops.push(t.clone());
			}
			TokType::LParen => ops.push(t.clone()),
			TokType::RParen => {
				let mut found = false;
				while let Some(top) = ops.pop() {
					if top.kind == TokType::LParen {
						found = true;
						break;
					}
					out.push(top);
				}
				if !found {
					return Err("Mismatched parentheses".into());
				}
			}
		}
		prev = Some(t);
	}
	while let Some(top) = ops.pop() {
		if top.kind == TokType::LParen || top.kind == TokType::RParen {
			return Err("Mismatched parentheses".into());
		}
		out.push(top);
	}
	Ok(out)
}

fn eval_rpn(rpn: &[Token]) -> Result<f64, String> {
	let mut stack: Vec<f64> = Vec::new();
	for t in rpn {
		match t.kind {
			TokType::Num => {
				let v: f64 = t.value.parse().map_err(|_| "Invalid number".to_string())?;
				stack.push(v);
			}
			TokType::Op => {
				let b = stack.pop().ok_or("Invalid expression")?;
				let a = stack.pop().ok_or("Invalid expression")?;
				let r = match t.value.as_str() {
					"+" => a + b,
					"-" => a - b,
					"*" => a * b,
					"/" => {
						if b == 0.0 {
							return Err("Division by zero".into());
						}
						a / b
					}
					"%" => {
						if b == 0.0 {
							return Err("Division by zero".into());
						}
						a % b
					}
					_ => return Err(format!("Unknown op: {}", t.value)),
				};
				stack.push(r);
			}
			_ => return Err("Invalid token in RPN".into()),
		}
	}
	if stack.len() != 1 {
		return Err("Invalid expression".into());
	}
	let result = stack.pop().unwrap();
	if !result.is_finite() {
		return Err("Math error".into());
	}
	Ok((result * 1e12).round() / 1e12)
}

pub fn evaluate(input: &str) -> Result<f64, String> {
	let trimmed = input.trim();
	if trimmed.is_empty() {
		return Ok(0.0);
	}
	let tokens = tokenize(trimmed)?;
	let rpn = to_rpn(tokens)?;
	eval_rpn(&rpn)
}

pub fn format_result(n: f64) -> String {
	if !n.is_finite() {
		return "Error".into();
	}
	if n.abs() < 1e-10 {
		return "0".into();
	}
	let abs = n.abs();
	if abs >= 1e16 || abs < 1e-6 {
		let s = format!("{:.6e}", n);
		return s.replace("e", "e");
	}
	let fixed = format!("{:.10}", n);
	let parsed: f64 = fixed.parse().unwrap_or(n);
	parsed.to_string()
}

#[tauri::command]
pub fn eval_expression(expr: String) -> Result<String, String> {
	let n = evaluate(&expr)?;
	Ok(format_result(n))
}
