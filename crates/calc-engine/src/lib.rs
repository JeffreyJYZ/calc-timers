//! Scientific calculator engine.
//!
//! Compiles to both rlib (for native tests and embed) and cdylib (for wasm-pack).
//! All math goes through `evaluate(input, angle_mode)`. `try_evaluate` swallows
//! errors and returns `Option<f64>` for safe preview calls.

use std::f64::consts::{E, PI};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
const TWO_PI: f64 = 2.0 * PI;
#[allow(dead_code)]
const HALF_PI: f64 = PI / 2.0;
const DEG_TO_RAD: f64 = PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / PI;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AngleMode {
    Rad,
    Deg,
}

impl Default for AngleMode {
    fn default() -> Self {
        AngleMode::Deg
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),
    Bang,
    LParen,
    RParen,
    Comma,
    Ident(String),
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_cont(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[allow(dead_code)]
const ENG_SUFFIXES: &[char] = &['k', 'K', 'M', 'G', 'T', 'P', 'm', 'u', 'n', 'p', 'f'];

fn eng_multiplier(c: char) -> Option<f64> {
    match c {
        'k' | 'K' => Some(1e3),
        'M' => Some(1e6),
        'G' => Some(1e9),
        'T' => Some(1e12),
        'P' => Some(1e15),
        'm' => Some(1e-3),
        'u' => Some(1e-6),
        'n' => Some(1e-9),
        'p' => Some(1e-12),
        'f' => Some(1e-15),
        _ => None,
    }
}

fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let s: String = input.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = s.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if is_digit(c) || c == '.' {
            let (num_str, end) = read_number(&chars, i)?;
            let parsed: f64 = num_str
                .parse()
                .map_err(|_| format!("Invalid number: {}", num_str))?;
            tokens.push(Tok::Num(parsed));
            i = end;
            // implicit multiplication: number followed by ident or '('
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() {
                let nc = chars[j];
                if is_ident_start(nc) || nc == '(' {
                    tokens.push(Tok::Op('*'));
                }
            }
            continue;
        }
        if is_ident_start(c) {
            let start = i;
            while i < chars.len() && is_ident_cont(chars[i]) {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            let is_fn = is_function(&name);
            tokens.push(Tok::Ident(name));
            // implicit multiplication: ident followed by '('
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '(' && !is_fn {
                tokens.push(Tok::Op('*'));
            }
            continue;
        }
        match c {
            '+' | '-' | '*' | '/' | '%' | '^' => {
                tokens.push(Tok::Op(c));
                i += 1;
            }
            '(' => {
                tokens.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Tok::RParen);
                i += 1;
                // implicit multiplication: ')' followed by ident or '('
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() {
                    let nc = chars[j];
                    if is_ident_start(nc) || nc == '(' {
                        tokens.push(Tok::Op('*'));
                    }
                }
            }
            ',' => {
                tokens.push(Tok::Comma);
                i += 1;
            }
            '!' => {
                tokens.push(Tok::Bang);
                i += 1;
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}

fn read_number(chars: &[char], start: usize) -> Result<(String, usize), String> {
    let mut i = start;
    let mut has_digit = false;
    let mut dots = 0;
    while i < chars.len() {
        let c = chars[i];
        if is_digit(c) {
            has_digit = true;
            i += 1;
        } else if c == '.' {
            dots += 1;
            if dots > 1 {
                return Err("Invalid number: multiple decimals".into());
            }
            i += 1;
        } else {
            break;
        }
    }
    if !has_digit {
        return Err("Invalid number".into());
    }
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        let exp_start = i;
        let mut j = i + 1;
        if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
            j += 1;
        }
        let mut exp_has_digit = false;
        while j < chars.len() && is_digit(chars[j]) {
            exp_has_digit = true;
            j += 1;
        }
        if exp_has_digit {
            i = j;
        } else {
            let _ = exp_start;
        }
    }
    if i < chars.len() {
        if let Some(mult) = eng_multiplier(chars[i]) {
            let after = if i + 1 < chars.len() {
                chars[i + 1]
            } else {
                '\0'
            };
            let valid_after = (after == '\0'
                || after == ')'
                || after == ','
                || after.is_whitespace()
                || "+-*/%^!".contains(after))
                && !is_ident_start(after)
                && !is_digit(after);
            if valid_after {
                let prefix: String = chars[start..i].iter().collect();
                let n: f64 = prefix
                    .parse()
                    .map_err(|_| format!("Invalid number: {}", prefix))?;
                let combined = n * mult;
                return Ok((format!("{}", combined), i + 1));
            }
        }
    }
    let num_str: String = chars[start..i].iter().collect();
    Ok((num_str, i))
}

fn precedence(op: char) -> i32 {
    match op {
        '+' | '-' => 1,
        '*' | '/' | '%' => 2,
        'y' => 3,
        '^' => 4,
        _ => 0,
    }
}

fn is_right_assoc(op: char) -> bool {
    op == '^' || op == 'y'
}

#[allow(dead_code)]
fn is_unary_prev(prev: Option<&Tok>) -> bool {
    match prev {
        None => true,
        Some(Tok::Op(_)) => true,
        Some(Tok::Bang) => true,
        Some(Tok::LParen) => true,
        Some(Tok::Comma) => true,
        _ => false,
    }
}

fn is_constant(name: &str) -> bool {
    matches!(name, "pi" | "e" | "PI" | "E")
}

fn const_value(name: &str) -> Option<f64> {
    match name {
        "pi" | "PI" => Some(PI),
        "e" | "E" => Some(std::f64::consts::E),
        _ => None,
    }
}

fn is_function(name: &str) -> bool {
    matches!(
        name,
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "log"
            | "ln"
            | "exp"
            | "sqrt"
            | "cbrt"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
            | "mod"
            | "pow"
            | "nPr"
            | "nCr"
            | "yroot"
            | "random"
            | "fac"
    )
}

fn function_arity(name: &str) -> usize {
    match name {
        "random" | "pi" | "e" | "PI" | "E" => 0,
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh" | "asinh"
        | "acosh" | "atanh" | "log" | "ln" | "exp" | "sqrt" | "cbrt" | "abs" | "floor"
        | "ceil" | "round" | "fac" => 1,
        _ => 2,
    }
}

fn to_rpn(tokens: Vec<Tok>) -> Result<Vec<Tok>, String> {
    let mut out: Vec<Tok> = Vec::new();
    let mut ops: Vec<Tok> = Vec::new();
    let n = tokens.len();
    let mut i = 0;
    #[derive(Clone)]
    struct Call {
        name: String,
        arity: usize,
        commas: usize,
        non_empty: bool,
        depth: usize,
    }
    let mut calls: Vec<Call> = Vec::new();
    let mut paren_depth: usize = 0;
    let mut chain_open = true;
    let mut paren_neg_stack: Vec<bool> = Vec::new();
    let mut paren_out_lens: Vec<usize> = Vec::new();

    while i < n {
        let t = tokens[i].clone();

        let mut sign: i32 = 1;
        let mut has_sign = false;
        if chain_open {
            while i < n {
                if let Tok::Op(c) = &tokens[i] {
                    if *c == '+' || *c == '-' {
                        if *c == '-' {
                            sign = -sign;
                        }
                        has_sign = true;
                        i += 1;
                        continue;
                    }
                }
                break;
            }
            if i >= n {
                break;
            }
        }
        let t = tokens[i].clone();

        match &t {
            Tok::Num(v) => {
                if has_sign && sign < 0 {
                    out.push(Tok::Num(0.0));
                    out.push(Tok::Num(*v));
                    out.push(Tok::Op('-'));
                } else {
                    out.push(Tok::Num(*v));
                }
                chain_open = false;
                if let Some(c) = calls.last_mut() {
                    c.non_empty = true;
                }
            }
            Tok::Ident(name) => {
                if is_constant(name) {
                    if has_sign && sign < 0 {
                        out.push(Tok::Num(0.0));
                        out.push(Tok::Ident(name.clone()));
                        out.push(Tok::Op('-'));
                    } else {
                        out.push(Tok::Ident(name.clone()));
                    }
                    chain_open = false;
                    if let Some(c) = calls.last_mut() {
                        c.non_empty = true;
                    }
                } else if is_function(name) {
                    if has_sign && sign < 0 {
                        return Err(format!("Cannot negate function {}", name));
                    }
                    if i + 1 >= n || !matches!(tokens[i + 1], Tok::LParen) {
                        return Err(format!("Function {} requires parentheses", name));
                    }
                    ops.push(t.clone());
                    chain_open = false;
                } else {
                    return Err(format!("Unknown identifier: {}", name));
                }
            }
            Tok::Op(c) => {
                while let Some(top) = ops.last().cloned() {
                    if let Tok::Op(tc) = top {
                        let p_top = precedence(tc);
                        let p_cur = precedence(*c);
                        if p_top == 0 {
                            break;
                        }
                        if p_top > p_cur || (p_top == p_cur && !is_right_assoc(*c)) {
                            out.push(ops.pop().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                ops.push(t.clone());
                chain_open = true;
            }
            Tok::Bang => {
                out.push(t.clone());
                chain_open = false;
                if let Some(c) = calls.last_mut() {
                    c.non_empty = true;
                }
            }
            Tok::LParen => {
                if has_sign && sign < 0 {
                    out.push(Tok::Num(0.0));
                    paren_neg_stack.push(true);
                } else {
                    paren_neg_stack.push(false);
                }
                if let Some(Tok::Ident(name)) = ops.last().cloned() {
                    if is_function(&name) {
                        let arity = function_arity(&name);
                        calls.push(Call {
                            name,
                            arity,
                            commas: 0,
                            non_empty: false,
                            depth: paren_depth + 1,
                        });
                        ops.pop();
                    }
                }
                paren_out_lens.push(out.len());
                ops.push(t.clone());
                paren_depth += 1;
                chain_open = true;
            }
            Tok::RParen => {
                if paren_depth == 0 {
                    return Err("Mismatched parentheses".into());
                }
                let paren_start = paren_out_lens.pop().unwrap_or(0);
                let pending_neg = paren_neg_stack.pop().unwrap_or(false);
                let pre_depth = paren_depth;
                let mut found = false;
                while let Some(top) = ops.last().cloned() {
                    if matches!(top, Tok::LParen) {
                        found = true;
                        break;
                    }
                    out.push(ops.pop().unwrap());
                }
                if !found {
                    return Err("Mismatched parentheses".into());
                }
                ops.pop();
                paren_depth -= 1;

                let is_fn_close = calls
                    .last()
                    .map(|c| c.depth == pre_depth)
                    .unwrap_or(false);

                if is_fn_close {
                    let call = calls.last().cloned().unwrap();
                    if call.commas > 0 && !call.non_empty {
                        return Err("Trailing comma in arguments".into());
                    }
                    let args = call.commas + if call.non_empty { 1 } else { 0 };
                    if args > call.arity {
                        return Err(format!(
                            "Too many arguments to {}: expected {}, got {}",
                            call.name, call.arity, args
                        ));
                    }
                    let mut provided = args;
                    while provided < call.arity {
                        out.push(Tok::Num(f64::NAN));
                        provided += 1;
                    }
                    out.push(Tok::Ident(call.name));
                    calls.pop();
                    if pending_neg {
                        out.push(Tok::Op('-'));
                    }
                } else if out.len() == paren_start && !pending_neg {
                    out.push(Tok::Num(0.0));
                } else if pending_neg {
                    out.push(Tok::Op('-'));
                }

                chain_open = false;
                if let Some(c) = calls.last_mut() {
                    c.non_empty = true;
                }
            }
            Tok::Comma => {
                let mut found = false;
                while let Some(top) = ops.last().cloned() {
                    if matches!(top, Tok::LParen) {
                        found = true;
                        break;
                    }
                    out.push(ops.pop().unwrap());
                }
                if !found {
                    return Err("Misplaced comma".into());
                }
                if let Some(c) = calls.last_mut() {
                    if c.non_empty {
                        c.commas += 1;
                        c.non_empty = false;
                    } else {
                        return Err("Empty argument".into());
                    }
                }
                chain_open = true;
            }
        }
        i += 1;
    }

    if chain_open && !ops.is_empty() {
        return Err("Incomplete expression".into());
    }
    if chain_open && ops.is_empty() && calls.is_empty() {
        out.push(Tok::Num(0.0));
    }

    while let Some(top) = ops.pop() {
        if matches!(top, Tok::LParen | Tok::RParen) {
            return Err("Mismatched parentheses".into());
        }
        out.push(top);
    }
    Ok(out)
}

fn fact_i64(n: i64) -> Result<f64, String> {
    if n < 0 {
        return Err("Factorial of negative".into());
    }
    if n > 170 {
        return Err("Factorial overflow".into());
    }
    let mut r: f64 = 1.0;
    for i in 2..=n {
        r *= i as f64;
    }
    Ok(r)
}

fn npr(n: f64, k: f64) -> Result<f64, String> {
    let ni = n as i64;
    let ki = k as i64;
    if ni < 0 || ki < 0 || ki > ni {
        return Err("Invalid nPr".into());
    }
    Ok(fact_i64(ni)? / fact_i64(ni - ki)?)
}

fn ncr(n: f64, k: f64) -> Result<f64, String> {
    let ni = n as i64;
    let ki = k as i64;
    if ni < 0 || ki < 0 || ki > ni {
        return Err("Invalid nCr".into());
    }
    let a = fact_i64(ni)?;
    let b = fact_i64(ki)?;
    let c = fact_i64(ni - ki)?;
    Ok(a / (b * c))
}

fn yroot(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Zero root degree".into());
    }
    Ok(a.powf(1.0 / b))
}

fn call_function(name: &str, args: &[f64], mode: AngleMode) -> Result<f64, String> {
    if let Some(v) = const_value(name) {
        if !args.is_empty() {
            return Err(format!("{} takes no arguments", name));
        }
        return Ok(v);
    }
    match name {
        "random" => {
            if !args.is_empty() {
                return Err("random takes no arguments".into());
            }
            Ok(random())
        }
        "sin" => {
            let a = args[0];
            let v = if mode == AngleMode::Deg {
                a * DEG_TO_RAD
            } else {
                a
            };
            Ok(v.sin())
        }
        "cos" => {
            let a = args[0];
            let v = if mode == AngleMode::Deg {
                a * DEG_TO_RAD
            } else {
                a
            };
            Ok(v.cos())
        }
        "tan" => {
            let a = args[0];
            let v = if mode == AngleMode::Deg {
                a * DEG_TO_RAD
            } else {
                a
            };
            Ok(v.tan())
        }
        "asin" => {
            let a = args[0];
            if !(-1.0..=1.0).contains(&a) {
                return Err("asin domain error".into());
            }
            let v = a.asin();
            Ok(if mode == AngleMode::Deg {
                v * RAD_TO_DEG
            } else {
                v
            })
        }
        "acos" => {
            let a = args[0];
            if !(-1.0..=1.0).contains(&a) {
                return Err("acos domain error".into());
            }
            let v = a.acos();
            Ok(if mode == AngleMode::Deg {
                v * RAD_TO_DEG
            } else {
                v
            })
        }
        "atan" => {
            let v = args[0].atan();
            Ok(if mode == AngleMode::Deg {
                v * RAD_TO_DEG
            } else {
                v
            })
        }
        "sinh" => Ok(args[0].sinh()),
        "cosh" => Ok(args[0].cosh()),
        "tanh" => Ok(args[0].tanh()),
        "asinh" => Ok(args[0].asinh()),
        "acosh" => {
            if args[0] < 1.0 {
                return Err("acosh domain error".into());
            }
            Ok(args[0].acosh())
        }
        "atanh" => {
            if !(-1.0..=1.0).contains(&args[0]) || args[0].abs() == 1.0 {
                return Err("atanh domain error".into());
            }
            Ok(args[0].atanh())
        }
        "log" => {
            if args[0] <= 0.0 {
                return Err("log domain error".into());
            }
            Ok(args[0].log10())
        }
        "ln" => {
            if args[0] <= 0.0 {
                return Err("ln domain error".into());
            }
            Ok(args[0].ln())
        }
        "exp" => Ok(args[0].exp()),
        "sqrt" => {
            if args[0] < 0.0 {
                return Err("sqrt domain error".into());
            }
            Ok(args[0].sqrt())
        }
        "cbrt" => Ok(args[0].cbrt()),
        "abs" => Ok(args[0].abs()),
        "floor" => Ok(args[0].floor()),
        "ceil" => Ok(args[0].ceil()),
        "round" => Ok(args[0].round()),
        "fac" => {
            let n = args[0].round() as i64;
            fact_i64(n)
        }
        "min" => {
            if args.len() != 2 {
                return Err("min takes 2 arguments".into());
            }
            Ok(args[0].min(args[1]))
        }
        "max" => {
            if args.len() != 2 {
                return Err("max takes 2 arguments".into());
            }
            Ok(args[0].max(args[1]))
        }
        "mod" => {
            if args.len() != 2 {
                return Err("mod takes 2 arguments".into());
            }
            if args[1] == 0.0 {
                return Err("mod by zero".into());
            }
            Ok(args[0] - (args[0] / args[1]).floor() * args[1])
        }
        "pow" => {
            if args.len() != 2 {
                return Err("pow takes 2 arguments".into());
            }
            Ok(args[0].powf(args[1]))
        }
        "nPr" => {
            if args.len() != 2 {
                return Err("nPr takes 2 arguments".into());
            }
            npr(args[0], args[1])
        }
        "nCr" => {
            if args.len() != 2 {
                return Err("nCr takes 2 arguments".into());
            }
            ncr(args[0], args[1])
        }
        "yroot" => {
            if args.len() != 2 {
                return Err("yroot takes 2 arguments".into());
            }
            yroot(args[0], args[1])
        }
        _ => Err(format!("Unknown function: {}", name)),
    }
}

#[cfg(target_arch = "wasm32")]
fn random() -> f64 {
    js_sys::Math::random()
}

#[cfg(not(target_arch = "wasm32"))]
fn random() -> f64 {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};
    thread_local! {
        static STATE: Cell<u64> = Cell::new({
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xdead_beef_cafe_babe)
        });
    }
    STATE.with(|s| {
        let mut x = s.get();
        if x == 0 {
            x = 0x1234_5678_9abc_def0;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

fn apply_op(op: char, a: f64, b: f64) -> Result<f64, String> {
    match op {
        '+' => Ok(a + b),
        '-' => Ok(a - b),
        '*' => Ok(a * b),
        '/' => {
            if b == 0.0 {
                return Err("Division by zero".into());
            }
            Ok(a / b)
        }
        '%' => {
            if b == 0.0 {
                return Err("Division by zero".into());
            }
            Ok(a % b)
        }
        '^' => Ok(a.powf(b)),
        'y' => yroot(a, b),
        _ => Err(format!("Unknown operator: {}", op)),
    }
}

pub(crate) fn eval_rpn(rpn: &[Tok], mode: AngleMode) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();
    for t in rpn {
        match t {
            Tok::Num(n) => stack.push(*n),
            Tok::Bang => {
                let a = stack
                    .pop()
                    .ok_or_else(|| "Invalid expression".to_string())?;
                let n = a.round() as i64;
                stack.push(fact_i64(n)?);
            }
            Tok::Op(c) => {
                let b = stack
                    .pop()
                    .ok_or_else(|| "Invalid expression".to_string())?;
                let a = stack
                    .pop()
                    .ok_or_else(|| "Invalid expression".to_string())?;
                stack.push(apply_op(*c, a, b)?);
            }
            Tok::Ident(name) => {
                let arity = function_arity(name);
                if stack.len() < arity {
                    return Err(format!("Missing arguments for {}", name));
                }
                let start = stack.len() - arity;
                let args = &stack[start..];
                let result = call_function(name, args, mode)?;
                stack.truncate(start);
                stack.push(result);
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
    Ok(result)
}

pub fn evaluate_internal(input: &str, mode: AngleMode) -> Result<f64, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(0.0);
    }
    let tokens = tokenize(trimmed)?;
    let rpn = to_rpn(tokens)?;
    eval_rpn(&rpn, mode)
}

#[wasm_bindgen]
pub fn evaluate(input: &str, mode: JsValue) -> Result<f64, JsError> {
    let mode: AngleMode = serde_wasm_bindgen::from_value(mode)
        .map_err(|e| JsError::new(&format!("Bad angle mode: {}", e)))?;
    evaluate_internal(input, mode).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn try_evaluate(input: &str, mode: JsValue) -> Option<f64> {
    let mode: AngleMode = match serde_wasm_bindgen::from_value(mode) {
        Ok(m) => m,
        Err(_) => AngleMode::default(),
    };
    try_evaluate_internal(input, mode)
}

pub fn try_evaluate_internal(input: &str, mode: AngleMode) -> Option<f64> {
    evaluate_internal(input, mode).ok()
}

#[wasm_bindgen]
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
        return s;
    }
    let fixed = format!("{:.10}", n);
    let parsed: f64 = fixed.parse().unwrap_or(n);
    parsed.to_string()
}

pub fn format_result_pub(n: f64) -> String {
    format_result(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    fn ev(input: &str) -> f64 {
        evaluate_internal(input, AngleMode::Rad).unwrap()
    }

    fn evd(input: &str) -> f64 {
        evaluate_internal(input, AngleMode::Deg).unwrap()
    }

    fn err(input: &str) -> String {
        evaluate_internal(input, AngleMode::Rad)
            .err()
            .unwrap_or_default()
    }

    // ---- basic arithmetic ----

    #[test]
    fn empty() {
        assert_eq!(ev(""), 0.0);
        assert_eq!(ev("   "), 0.0);
    }

    #[test]
    fn single_number_int() {
        assert_eq!(ev("0"), 0.0);
        assert_eq!(ev("1"), 1.0);
        assert_eq!(ev("42"), 42.0);
        assert_eq!(ev("1000000"), 1_000_000.0);
    }

    #[test]
    fn single_number_decimal() {
        assert_eq!(ev("0.5"), 0.5);
        assert_eq!(ev("3.14159"), 3.14159);
        assert_eq!(ev(".5"), 0.5);
    }

    #[test]
    fn number_leading_decimal() {
        assert_eq!(ev(".5"), 0.5);
        assert_eq!(ev(".25+.75"), 1.0);
    }

    #[test]
    fn number_sci_notation() {
        assert_eq!(ev("1e3"), 1000.0);
        assert_eq!(ev("2E3"), 2000.0);
        assert_eq!(ev("1.5e2"), 150.0);
        assert_eq!(ev("1e-3"), 0.001);
        assert_eq!(ev("2.5E+2"), 250.0);
    }

    #[test]
    fn add() {
        assert_eq!(ev("1+1"), 2.0);
        assert_eq!(ev("1+2+3"), 6.0);
        assert_eq!(ev("0+0"), 0.0);
        assert_eq!(ev("1.5+2.5"), 4.0);
        assert_eq!(ev("100+200+300"), 600.0);
    }

    #[test]
    fn sub() {
        assert_eq!(ev("5-3"), 2.0);
        assert_eq!(ev("10-5-3"), 2.0);
        assert_eq!(ev("0-1"), -1.0);
        assert_eq!(ev("1-1"), 0.0);
    }

    #[test]
    fn mul() {
        assert_eq!(ev("2*3"), 6.0);
        assert_eq!(ev("2*3*4"), 24.0);
        assert_eq!(ev("0*999"), 0.0);
        assert_eq!(ev("-2*3"), -6.0);
    }

    #[test]
    fn div() {
        assert_eq!(ev("6/2"), 3.0);
        assert_eq!(ev("10/4"), 2.5);
        assert_eq!(ev("0/5"), 0.0);
    }

    #[test]
    fn div_by_zero() {
        assert!(err("1/0").contains("Division by zero"));
        assert!(err("5%0").contains("Division by zero"));
    }

    #[test]
    fn mod_op() {
        assert_eq!(ev("7%3"), 1.0);
        assert_eq!(ev("10%4"), 2.0);
        assert_eq!(ev("5%5"), 0.0);
    }

    #[test]
    fn precedence_mul_over_add() {
        assert_eq!(ev("2+3*4"), 14.0);
        assert_eq!(ev("2*3+4"), 10.0);
        assert_eq!(ev("2+3*4+5"), 19.0);
    }

    #[test]
    fn parens() {
        assert_eq!(ev("(2+3)*4"), 20.0);
        assert_eq!(ev("((2+3)*4)-5"), 15.0);
        assert_eq!(ev("(((1)))"), 1.0);
    }

    #[test]
    fn unary_minus() {
        assert_eq!(ev("-5"), -5.0);
        assert_eq!(ev("--5"), 5.0);
        assert_eq!(ev("3*-2"), -6.0);
        assert_eq!(ev("-(2+3)"), -5.0);
        assert_eq!(ev("-(2^2)"), -4.0);
        assert_eq!(ev("(-2)^2"), 4.0);
    }

    #[test]
    fn unary_plus() {
        assert_eq!(ev("+5"), 5.0);
        assert_eq!(ev("++5"), 5.0);
    }

    #[test]
    fn paren_after_unary() {
        assert_eq!(ev("-(3)"), -3.0);
        assert_eq!(ev("(-3)"), -3.0);
    }

    // ---- power ----

    #[test]
    fn power_basic() {
        assert_eq!(ev("2^3"), 8.0);
        assert_eq!(ev("2^10"), 1024.0);
        assert_eq!(ev("5^2"), 25.0);
    }

    #[test]
    fn power_right_assoc() {
        assert_eq!(ev("2^3^2"), 512.0);
    }

    #[test]
    fn power_with_parens() {
        assert_eq!(ev("(2+1)^2"), 9.0);
        assert_eq!(ev("2^(3+1)"), 16.0);
    }

    #[test]
    fn power_fractional() {
        assert!(approx(ev("4^0.5"), 2.0, 1e-9));
        assert!(approx(ev("27^(1/3)"), 3.0, 1e-9));
    }

    #[test]
    fn pow_function() {
        assert_eq!(ev("pow(2,10)"), 1024.0);
        assert_eq!(ev("pow(3,2)"), 9.0);
    }

    // ---- factorial ----

    #[test]
    fn factorial_basic() {
        assert_eq!(ev("0!"), 1.0);
        assert_eq!(ev("1!"), 1.0);
        assert_eq!(ev("2!"), 2.0);
        assert_eq!(ev("3!"), 6.0);
        assert_eq!(ev("4!"), 24.0);
        assert_eq!(ev("5!"), 120.0);
        assert_eq!(ev("6!"), 720.0);
        assert_eq!(ev("10!"), 3_628_800.0);
    }

    #[test]
    fn factorial_in_expr() {
        assert_eq!(ev("2+3!"), 8.0);
        assert_eq!(ev("(2+3)!"), 120.0);
    }

    #[test]
    fn factorial_negative_errors() {
        assert!(err("(-1)!").contains("Factorial"));
    }

    #[test]
    fn factorial_fac_fn() {
        assert_eq!(ev("fac(5)"), 120.0);
    }

    // ---- constants ----

    #[test]
    fn pi() {
        assert!(approx(ev("pi"), PI, 1e-9));
        assert!(approx(ev("pi*2"), TWO_PI, 1e-9));
    }

    #[test]
    fn euler() {
        assert!(approx(ev("e"), E, 1e-9));
    }

    // ---- trig rad ----

    #[test]
    fn sin_rad() {
        assert!(approx(ev("sin(0)"), 0.0, 1e-9));
        assert!(approx(ev("sin(pi/2)"), 1.0, 1e-9));
        assert!(approx(ev("sin(pi)"), 0.0, 1e-9));
    }

    #[test]
    fn cos_rad() {
        assert!(approx(ev("cos(0)"), 1.0, 1e-9));
        assert!(approx(ev("cos(pi/2)"), 0.0, 1e-9));
        assert!(approx(ev("cos(pi)"), -1.0, 1e-9));
    }

    #[test]
    fn tan_rad() {
        assert!(approx(ev("tan(0)"), 0.0, 1e-9));
        assert!(approx(ev("tan(pi/4)"), 1.0, 1e-9));
    }

    // ---- trig deg ----

    #[test]
    fn sin_deg() {
        assert!(approx(evd("sin(0)"), 0.0, 1e-9));
        assert!(approx(evd("sin(30)"), 0.5, 1e-9));
        assert!(approx(evd("sin(90)"), 1.0, 1e-9));
        assert!(approx(evd("sin(180)"), 0.0, 1e-9));
    }

    #[test]
    fn cos_deg() {
        assert!(approx(evd("cos(0)"), 1.0, 1e-9));
        assert!(approx(evd("cos(60)"), 0.5, 1e-9));
        assert!(approx(evd("cos(90)"), 0.0, 1e-9));
    }

    #[test]
    fn tan_deg() {
        assert!(approx(evd("tan(45)"), 1.0, 1e-9));
        assert!(approx(evd("tan(0)"), 0.0, 1e-9));
    }

    #[test]
    fn asin_deg() {
        assert!(approx(evd("asin(0.5)"), 30.0, 1e-9));
        assert!(approx(evd("asin(1)"), 90.0, 1e-9));
    }

    #[test]
    fn acos_deg() {
        assert!(approx(evd("acos(0.5)"), 60.0, 1e-9));
        assert!(approx(evd("acos(0)"), 90.0, 1e-9));
    }

    #[test]
    fn atan_deg() {
        assert!(approx(evd("atan(1)"), 45.0, 1e-9));
        assert!(approx(evd("atan(0)"), 0.0, 1e-9));
    }

    // ---- hyperbolic ----

    #[test]
    fn sinh_cosh_tanh() {
        assert!(approx(ev("sinh(0)"), 0.0, 1e-9));
        assert!(approx(ev("cosh(0)"), 1.0, 1e-9));
        assert!(approx(ev("tanh(0)"), 0.0, 1e-9));
        assert!(approx(ev("sinh(1)"), 1.0_f64.sinh(), 1e-9));
    }

    // ---- log/exp ----

    #[test]
    fn ln() {
        assert!(approx(ev("ln(1)"), 0.0, 1e-9));
        assert!(approx(ev("ln(e)"), 1.0, 1e-9));
        assert!(approx(ev("ln(e^3)"), 3.0, 1e-9));
    }

    #[test]
    fn log10() {
        assert!(approx(ev("log(1)"), 0.0, 1e-9));
        assert!(approx(ev("log(10)"), 1.0, 1e-9));
        assert!(approx(ev("log(1000)"), 3.0, 1e-9));
        assert!(approx(ev("log(100)"), 2.0, 1e-9));
    }

    #[test]
    fn exp_fn() {
        assert!(approx(ev("exp(0)"), 1.0, 1e-9));
        assert!(approx(ev("exp(1)"), E, 1e-9));
        assert!(approx(ev("exp(ln(2))"), 2.0, 1e-9));
    }

    // ---- roots ----

    #[test]
    fn sqrt_fn() {
        assert!(approx(ev("sqrt(0)"), 0.0, 1e-9));
        assert!(approx(ev("sqrt(1)"), 1.0, 1e-9));
        assert!(approx(ev("sqrt(4)"), 2.0, 1e-9));
        assert!(approx(ev("sqrt(2)"), 1.4142135623730951, 1e-9));
    }

    #[test]
    fn cbrt_fn() {
        assert!(approx(ev("cbrt(8)"), 2.0, 1e-9));
        assert!(approx(ev("cbrt(27)"), 3.0, 1e-9));
        assert!(approx(ev("cbrt(-8)"), -2.0, 1e-9));
    }

    #[test]
    fn nth_root() {
        assert!(approx(ev("yroot(8,3)"), 2.0, 1e-9));
        assert!(approx(ev("yroot(27,3)"), 3.0, 1e-9));
        assert!(approx(ev("yroot(16,2)"), 4.0, 1e-9));
    }

    // ---- rounding ----

    #[test]
    fn abs_fn() {
        assert_eq!(ev("abs(0)"), 0.0);
        assert_eq!(ev("abs(-5)"), 5.0);
        assert_eq!(ev("abs(5)"), 5.0);
    }

    #[test]
    fn floor_ceil_round() {
        assert_eq!(ev("floor(1.7)"), 1.0);
        assert_eq!(ev("floor(-1.2)"), -2.0);
        assert_eq!(ev("ceil(1.2)"), 2.0);
        assert_eq!(ev("ceil(-1.7)"), -1.0);
        assert_eq!(ev("round(1.5)"), 2.0);
        assert_eq!(ev("round(1.4)"), 1.0);
    }

    // ---- min/max/mod ----

    #[test]
    fn min_max() {
        assert_eq!(ev("min(2,3)"), 2.0);
        assert_eq!(ev("min(-1,1)"), -1.0);
        assert_eq!(ev("max(2,3)"), 3.0);
        assert_eq!(ev("max(-1,1)"), 1.0);
    }

    #[test]
    fn mod_fn() {
        assert!(approx(ev("mod(7,3)"), 1.0, 1e-9));
        assert!(approx(ev("mod(10,4)"), 2.0, 1e-9));
        assert!(approx(ev("mod(-7,3)"), 2.0, 1e-9));
        assert!(approx(ev("mod(7,-3)"), -2.0, 1e-9));
    }

    // ---- combinatorics ----

    #[test]
    fn npr_basic() {
        assert_eq!(ev("nPr(5,2)"), 20.0);
        assert_eq!(ev("nPr(5,0)"), 1.0);
        assert_eq!(ev("nPr(5,5)"), 120.0);
        assert_eq!(ev("nPr(10,3)"), 720.0);
    }

    #[test]
    fn ncr_basic() {
        assert_eq!(ev("nCr(5,2)"), 10.0);
        assert_eq!(ev("nCr(5,0)"), 1.0);
        assert_eq!(ev("nCr(5,5)"), 1.0);
        assert_eq!(ev("nCr(10,3)"), 120.0);
    }

    // ---- engineering prefixes ----

    #[test]
    fn eng_kilo() {
        assert_eq!(ev("1k"), 1000.0);
        assert_eq!(ev("2.5k"), 2500.0);
        assert_eq!(ev("1K"), 1000.0);
    }

    #[test]
    fn eng_mega() {
        assert_eq!(ev("1M"), 1e6);
        assert_eq!(ev("2M"), 2e6);
    }

    #[test]
    fn eng_milli() {
        assert_eq!(ev("1m"), 1e-3);
        assert_eq!(ev("100m"), 0.1);
    }

    #[test]
    fn eng_micro_nano_pico() {
        assert_eq!(ev("1u"), 1e-6);
        assert_eq!(ev("1n"), 1e-9);
        assert_eq!(ev("1p"), 1e-12);
        assert_eq!(ev("1f"), 1e-15);
    }

    #[test]
    fn eng_in_expr() {
        assert_eq!(ev("1k+500"), 1500.0);
        assert_eq!(ev("2*3k"), 6000.0);
        assert_eq!(ev("1M/1k"), 1000.0);
    }

    // ---- domain errors ----

    #[test]
    fn ln_negative() {
        assert!(err("ln(-1)").contains("ln"));
    }

    #[test]
    fn ln_zero() {
        assert!(err("ln(0)").contains("ln"));
    }

    #[test]
    fn sqrt_negative() {
        assert!(err("sqrt(-1)").contains("sqrt"));
    }

    #[test]
    fn log_negative() {
        assert!(err("log(-1)").contains("log"));
    }

    #[test]
    fn asin_oor() {
        assert!(err("asin(2)").contains("asin"));
        assert!(err("asin(-2)").contains("asin"));
    }

    #[test]
    fn acos_oor() {
        assert!(err("acos(2)").contains("acos"));
    }

    #[test]
    fn mod_zero() {
        assert!(err("mod(5,0)").contains("mod"));
    }

    #[test]
    fn div_zero_in_expr() {
        assert!(err("1/(2-2)").contains("Division"));
    }

    // ---- paren mismatches ----

    #[test]
    fn extra_open_paren() {
        assert!(err("(1+2").contains("parentheses"));
    }

    #[test]
    fn extra_close_paren() {
        assert!(err("1+2)").contains("parentheses"));
    }

    // ---- whitespace and weird input ----

    #[test]
    fn whitespace_tolerated() {
        assert_eq!(ev("  1  +  2  "), 3.0);
        assert_eq!(ev("\t1\n+\t2"), 3.0);
    }

    #[test]
    fn unknown_ident() {
        assert!(err("foo(1)").contains("Unknown"));
        assert!(err("xyz").contains("Unknown"));
    }

    #[test]
    fn unknown_char() {
        assert!(err("1&2").contains("Unexpected"));
        assert!(err("1@2").contains("Unexpected"));
    }

    #[test]
    fn invalid_number() {
        assert!(err("1.2.3").contains("Invalid"));
    }

    // ---- formatting ----

    #[test]
    fn format_zero() {
        assert_eq!(format_result(0.0), "0");
    }

    #[test]
    fn format_int() {
        assert_eq!(format_result(42.0), "42");
        assert_eq!(format_result(-7.0), "-7");
    }

    #[test]
    fn format_decimal() {
        assert_eq!(format_result(0.5), "0.5");
        assert_eq!(format_result(1.25), "1.25");
    }

    #[test]
    fn format_large() {
        let s = format_result(1e20);
        assert!(s.contains("e"));
    }

    #[test]
    fn format_small() {
        let s = format_result(1e-10);
        assert!(s.contains("e") || s == "0");
    }

    #[test]
    fn format_nan() {
        assert_eq!(format_result(f64::NAN), "Error");
        assert_eq!(format_result(f64::INFINITY), "Error");
    }

    // ---- complex expressions ----

    #[test]
    fn quadratic_via_bhaskara() {
        let a: f64 = 1.0;
        let b: f64 = -5.0;
        let c: f64 = 6.0;
        let disc: f64 = b * b - 4.0 * a * c;
        let r1 = (-b + disc.sqrt()) / (2.0 * a);
        let r2 = (-b - disc.sqrt()) / (2.0 * a);
        assert_eq!(r1, 3.0);
        assert_eq!(r2, 2.0);
    }

    #[test]
    fn compound_expr() {
        assert_eq!(ev("(1+2)*(3+4)"), 21.0);
        assert_eq!(ev("2^3+4*5"), 28.0);
        assert_eq!(ev("(2+3)^(1+1)"), 25.0);
    }

    #[test]
    fn nested_fns() {
        assert!(approx(ev("sqrt(sqrt(16))"), 2.0, 1e-9));
        assert!(approx(ev("ln(exp(5))"), 5.0, 1e-9));
        assert!(approx(ev("exp(ln(7))"), 7.0, 1e-9));
    }

    #[test]
    fn trig_chain() {
        assert!(approx(ev("sin(0)+cos(0)"), 1.0, 1e-9));
        assert!(approx(ev("sin(pi/4)^2 + cos(pi/4)^2"), 1.0, 1e-9));
    }

    #[test]
    fn pi_in_trig() {
        assert!(approx(ev("sin(pi)"), 0.0, 1e-9));
        assert!(approx(ev("cos(2*pi)"), 1.0, 1e-9));
    }

    #[test]
    fn e_in_ln() {
        assert!(approx(ev("ln(e*e)"), 2.0, 1e-9));
    }

    // ---- try_evaluate ----

    #[test]
    fn try_eval_ok() {
        assert_eq!(try_evaluate_internal("1+1", AngleMode::Rad), Some(2.0));
    }

    #[test]
    fn try_eval_err() {
        assert_eq!(try_evaluate_internal("1/0", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("sqrt(-1)", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("(1+2", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("", AngleMode::Rad), Some(0.0));
    }

    // ---- mode persisted ----

    #[test]
    fn mode_changes_trig_result() {
        let r = evaluate_internal("sin(90)", AngleMode::Rad).unwrap();
        let d = evaluate_internal("sin(90)", AngleMode::Deg).unwrap();
        assert!(approx(r, 0.8939966636005579, 1e-9));
        assert!(approx(d, 1.0, 1e-9));
    }

    // ---- unary operator combinations ----

    #[test]
    fn double_unary() {
        assert_eq!(ev("--3"), 3.0);
        assert_eq!(ev("---3"), -3.0);
        assert_eq!(ev("-+-3"), 3.0);
    }

    #[test]
    fn unary_in_parens() {
        assert_eq!(ev("(-3)*(-3)"), 9.0);
    }

    // ---- comma / multi-arg ----

    #[test]
    fn min_max_with_exprs() {
        assert_eq!(ev("min(1+2, 3+4)"), 3.0);
        assert_eq!(ev("max(1+2, 3+4)"), 7.0);
    }

    // ---- power edge cases ----

    #[test]
    fn power_zero() {
        assert_eq!(ev("0^0"), 1.0);
        assert_eq!(ev("0^5"), 0.0);
    }

    #[test]
    fn power_neg_base() {
        assert_eq!(ev("(-2)^3"), -8.0);
        assert_eq!(ev("(-2)^2"), 4.0);
    }

    // ---- mismatched comma ----

    #[test]
    fn bare_comma() {
        assert!(err("1,2").contains("Misplaced") || err("1,2").contains("Unknown"));
    }

    // ---- big numbers ----

    #[test]
    fn big_factorial() {
        assert_eq!(ev("20!"), 2_432_902_008_176_640_000.0);
    }

    // ---- function followed by literal ----

    #[test]
    fn function_no_paren() {
        assert!(err("sin 5").contains("requires parentheses") || err("sin 5").contains("parentheses"));
    }

    // ---- deeply nested parens ----

    #[test]
    fn deep_nesting() {
        assert_eq!(ev("((((((1+2))))))"), 3.0);
    }

    // ---- combined tokens ----

    #[test]
    fn number_then_ident() {
        assert_eq!(ev("2pi"), TWO_PI);
        assert_eq!(ev("3pi"), 3.0 * PI);
    }

    #[test]
    fn multi_digit_after_e() {
        assert_eq!(ev("1e10"), 1e10);
    }

    #[test]
    fn no_paren_2arg() {
        // Functions need parens — bare ident after digits is constant
        assert!(approx(ev("2e"), 2.0 * E, 1e-9));
    }

    // ---- random is in [0, 1) ----

    #[test]
    fn random_range() {
        for _ in 0..50 {
            let r = ev("random()");
            assert!((0.0..1.0).contains(&r));
        }
    }

    // ---- large powers ----

    #[test]
    fn large_power() {
        assert!(approx(ev("2^40"), 1_099_511_627_776.0, 1.0));
    }

    // ---- inverse functions ----

    #[test]
    fn sin_asin_roundtrip_rad() {
        let v = 0.7_f64;
        let s = ev(&format!("sin({})", v));
        let b = ev(&format!("asin({})", s));
        assert!(approx(v, b, 1e-9));
    }

    // ---- paren in function chain ----

    #[test]
    fn nested_fns_three_levels() {
        assert!(approx(ev("abs(floor(-3.5))"), 4.0, 1e-9));
    }

    // ---- comma arg count ----

    #[test]
    fn too_many_args() {
        assert!(err("min(1,2,3)").contains("Too many"));
    }

    // ---- constant in expr ----

    #[test]
    fn pi_times_two() {
        assert!(approx(ev("pi*2"), TWO_PI, 1e-9));
    }

    #[test]
    fn e_squared() {
        assert!(approx(ev("e^2"), E * E, 1e-9));
    }

    // ---- sum of fractions ----

    #[test]
    fn fraction_sum() {
        assert!(approx(ev("1/2+1/3+1/6"), 1.0, 1e-9));
    }

    // ---- mod with negatives ----

    #[test]
    fn mod_negative_result() {
        assert!(approx(ev("mod(-7,3)"), 2.0, 1e-9));
        assert!(approx(ev("mod(7,-3)"), -2.0, 1e-9));
    }

    // ---- additional comprehensive tests ----

    #[test]
    fn chained_add() {
        assert_eq!(ev("1+2+3+4+5+6+7+8+9+10"), 55.0);
    }

    #[test]
    fn chained_mul() {
        assert_eq!(ev("1*2*3*4*5"), 120.0);
    }

    #[test]
    fn mixed_chain() {
        assert_eq!(ev("1+2*3-4/2+5"), 1.0 + 6.0 - 2.0 + 5.0);
    }

    #[test]
    fn long_unary() {
        assert_eq!(ev("-----5"), -5.0);
    }

    #[test]
    fn nested_calls() {
        assert!(approx(ev("max(min(1,2), min(3,4))"), 3.0, 1e-9));
    }

    #[test]
    fn trig_inverse_roundtrip() {
        let v = 0.5_f64;
        let a = ev(&format!("sin({})", v));
        let b = ev(&format!("asin({})", a));
        assert!(approx(v, b, 1e-9));
    }

    #[test]
    fn exp_ln_roundtrip() {
        let v = 123.456_f64;
        let a = ev(&format!("exp({})", v));
        let b = ev(&format!("ln({})", a));
        assert!(approx(v, b, 1e-9));
    }

    #[test]
    fn sqrt_square_roundtrip() {
        let v: f64 = 99.0;
        let b = ev(&format!("sqrt({})", v));
        let c = ev(&format!("sqrt({})", b));
        assert!(approx(c, v.sqrt().sqrt(), 1e-9));
    }

    #[test]
    fn cbrt_negative() {
        assert!(approx(ev("cbrt(-27)"), -3.0, 1e-9));
    }

    #[test]
    fn npr_5_3() {
        assert_eq!(ev("nPr(5,3)"), 60.0);
    }

    #[test]
    fn ncr_5_3() {
        assert_eq!(ev("nCr(5,3)"), 10.0);
    }

    #[test]
    fn npr_n_0() {
        assert_eq!(ev("nPr(7,0)"), 1.0);
    }

    #[test]
    fn ncr_n_n() {
        assert_eq!(ev("nCr(8,8)"), 1.0);
    }

    #[test]
    fn pi_e_product() {
        assert!(approx(ev("pi*e"), PI * E, 1e-9));
    }

    #[test]
    fn long_fraction() {
        assert!(approx(ev("22/7"), 22.0 / 7.0, 1e-12));
    }

    #[test]
    fn exp_5() {
        assert!(approx(ev("exp(5)"), 5.0_f64.exp(), 1e-9));
    }

    #[test]
    fn log_of_powers() {
        assert!(approx(ev("log(10^5)"), 5.0, 1e-9));
        assert!(approx(ev("log(10^10)"), 10.0, 1e-9));
    }

    #[test]
    fn ln_of_powers() {
        assert!(approx(ev("ln(e^7)"), 7.0, 1e-9));
    }

    #[test]
    fn neg_squared() {
        assert_eq!(ev("(-3)^2"), 9.0);
    }

    #[test]
    fn neg_cubed() {
        assert_eq!(ev("(-3)^3"), -27.0);
    }

    #[test]
    fn zero_pow_zero() {
        assert_eq!(ev("0^0"), 1.0);
    }

    #[test]
    fn one_pow_anything() {
        assert_eq!(ev("1^1000"), 1.0);
    }

    #[test]
    fn large_left_assoc() {
        assert_eq!(ev("100-50-25"), 25.0);
    }

    #[test]
    fn nested_factorial() {
        assert_eq!(ev("(3+2)!"), 120.0);
    }

    #[test]
    fn sin_cos_identity() {
        for x in [0.0, 0.1, 0.5, 1.0, 1.5, 2.0, 3.0] {
            let a = ev(&format!("sin({})^2 + cos({})^2", x, x));
            assert!(approx(a, 1.0, 1e-9), "failed for x={}", x);
        }
    }

    #[test]
    fn parens_in_pow() {
        assert_eq!(ev("(2+1)^(1+2)"), 27.0);
    }

    #[test]
    fn big_combo() {
        // 2 * (3 + 4) - 5 / 2 + 6^2 = 14 - 2.5 + 36 = 47.5
        assert!(approx(
            ev("2*(3+4)-5/2+6^2"),
            2.0 * 7.0 - 2.5 + 36.0,
            1e-9
        ));
    }

    #[test]
    fn mixed_unary_binary() {
        assert_eq!(ev("-2+-3"), -5.0);
    }

    #[test]
    fn unary_inside_call() {
        assert!(approx(ev("abs(-2-3)"), 5.0, 1e-9));
    }

    #[test]
    fn comma_no_space() {
        assert_eq!(ev("min(1,2)"), 1.0);
        assert_eq!(ev("max(3,4)"), 4.0);
    }

    #[test]
    fn trailing_comma_err() {
        assert!(err("min(1,)").contains("Trailing"));
    }

    #[test]
    fn leading_comma_err() {
        assert!(err("min(,1)").contains("Empty"));
    }

    #[test]
    fn function_with_0_arg() {
        let r = ev("pi");
        assert!(approx(r, PI, 1e-9));
    }

    #[test]
    fn empty_inside_parens() {
        assert_eq!(ev("()"), 0.0);
    }

    #[test]
    fn only_unary() {
        assert_eq!(ev("-"), 0.0);
    }

    #[test]
    fn trailing_dot() {
        assert_eq!(ev("5."), 5.0);
    }

    #[test]
    fn leading_dot() {
        assert_eq!(ev(".5"), 0.5);
    }

    #[test]
    fn many_decimals() {
        assert_eq!(ev("0.123456789"), 0.123456789);
    }

    #[test]
    fn long_int() {
        assert_eq!(ev("1234567890"), 1_234_567_890.0);
    }

    #[test]
    fn eng_giga() {
        assert_eq!(ev("1G"), 1e9);
    }

    #[test]
    fn eng_tera() {
        assert_eq!(ev("1T"), 1e12);
    }

    #[test]
    fn eng_peta() {
        assert_eq!(ev("1P"), 1e15);
    }

    #[test]
    fn eng_combo() {
        assert_eq!(ev("1k+1m"), 1_000.001);
    }

    #[test]
    fn factorial_large() {
        assert_eq!(ev("15!"), 1_307_674_368_000.0);
    }

    #[test]
    fn pow_with_sqrt() {
        assert!(approx(ev("sqrt(2)^2"), 2.0, 1e-9));
    }

    #[test]
    fn cubic_identity() {
        // (a+b)^3 = a^3 + 3a^2b + 3ab^2 + b^3
        let a: f64 = 2.0;
        let b: f64 = 3.0;
        let lhs = ev(&format!("({}+{})^3", a, b));
        let rhs = a.powi(3) + 3.0 * a.powi(2) * b + 3.0 * a * b.powi(2) + b.powi(3);
        assert!(approx(lhs, rhs, 1e-9));
    }

    #[test]
    fn diff_of_squares() {
        let a: f64 = 7.0;
        let b: f64 = 4.0;
        let lhs = ev(&format!("({})*({})", a + b, a - b));
        let rhs = a * a - b * b;
        assert!(approx(lhs, rhs, 1e-9));
    }

    #[test]
    fn sin_decomposition() {
        // sin(2x) = 2 sin(x) cos(x)
        let x: f64 = 0.7;
        let lhs = ev(&format!("sin(2*{})", x));
        let rhs = 2.0 * x.sin() * x.cos();
        assert!(approx(lhs, rhs, 1e-9));
    }

    #[test]
    fn cos_decomposition() {
        // cos(2x) = cos^2(x) - sin^2(x)
        let x: f64 = 0.4;
        let lhs = ev(&format!("cos(2*{})", x));
        let rhs = x.cos().powi(2) - x.sin().powi(2);
        assert!(approx(lhs, rhs, 1e-9));
    }

    #[test]
    fn tan_via_sin_cos() {
        let x: f64 = 0.9;
        let lhs = ev(&format!("tan({})", x));
        let rhs = x.sin() / x.cos();
        assert!(approx(lhs, rhs, 1e-9));
    }

    #[test]
    fn pow_int_exp() {
        assert_eq!(ev("2^0"), 1.0);
        assert_eq!(ev("2^1"), 2.0);
        assert_eq!(ev("2^2"), 4.0);
        assert_eq!(ev("2^3"), 8.0);
        assert_eq!(ev("2^4"), 16.0);
        assert_eq!(ev("2^5"), 32.0);
    }

    #[test]
    fn unary_chain_plus_minus() {
        assert_eq!(ev("+-+5"), -5.0);
        assert_eq!(ev("+-+-5"), 5.0);
    }

    #[test]
    fn long_chain_mixed() {
        assert_eq!(
            ev("1+2-3*4/5+6-7*8+9/10"),
            1.0 + 2.0 - 3.0 * 4.0 / 5.0 + 6.0 - 7.0 * 8.0 + 9.0 / 10.0
        );
    }

    #[test]
    fn deeply_nested() {
        // ((((1+2)*(3+4))+5)-6) = 3*7+5-6 = 20
        assert_eq!(ev("((((1+2)*(3+4))+5)-6)"), 20.0);
    }

    #[test]
    fn factorial_of_zero() {
        assert_eq!(ev("0!"), 1.0);
    }

    #[test]
    fn big_factorial_check() {
        // 12! = 479001600
        assert_eq!(ev("12!"), 479_001_600.0);
    }

    #[test]
    fn deg_rad_toggle_sin() {
        let r1 = evaluate_internal("sin(45)", AngleMode::Deg).unwrap();
        let r2 = evaluate_internal("sin(45)", AngleMode::Rad).unwrap();
        assert!(approx(r1, 0.7071067811865475, 1e-9));
        assert!(approx(r2, 0.8509035245341184, 1e-9));
    }

    #[test]
    fn deg_rad_toggle_cos() {
        let r1 = evaluate_internal("cos(30)", AngleMode::Deg).unwrap();
        let r2 = evaluate_internal("cos(30)", AngleMode::Rad).unwrap();
        assert!(approx(r1, 0.8660254037844387, 1e-9));
        assert!(approx(r2, 0.15425144988758405, 1e-9));
    }

    #[test]
    fn deg_rad_toggle_tan() {
        let r1 = evaluate_internal("tan(60)", AngleMode::Deg).unwrap();
        assert!(approx(r1, 1.7320508075688772, 1e-9));
    }

    #[test]
    fn atan_deg_basic() {
        let r = evaluate_internal("atan(1)", AngleMode::Deg).unwrap();
        assert!(approx(r, 45.0, 1e-9));
    }

    #[test]
    fn yroot_cube() {
        assert!(approx(ev("yroot(27,3)"), 3.0, 1e-9));
    }

    #[test]
    fn yroot_square() {
        assert!(approx(ev("yroot(25,2)"), 5.0, 1e-9));
    }

    #[test]
    fn yroot_fourth() {
        assert!(approx(ev("yroot(16,4)"), 2.0, 1e-9));
    }

    #[test]
    fn floor_neg() {
        assert_eq!(ev("floor(-2.3)"), -3.0);
    }

    #[test]
    fn ceil_neg() {
        assert_eq!(ev("ceil(-2.3)"), -2.0);
    }

    #[test]
    fn round_pos() {
        assert_eq!(ev("round(2.5)"), 3.0);
        assert_eq!(ev("round(2.4)"), 2.0);
    }

    #[test]
    fn round_neg() {
        assert_eq!(ev("round(-2.5)"), -3.0);
    }

    #[test]
    fn abs_pos_neg() {
        assert_eq!(ev("abs(7)"), 7.0);
        assert_eq!(ev("abs(-7)"), 7.0);
        assert_eq!(ev("abs(0)"), 0.0);
    }

    #[test]
    fn sinh_pos() {
        assert!(approx(ev("sinh(1)"), 1.1752011936438014, 1e-9));
    }

    #[test]
    fn cosh_pos() {
        assert!(approx(ev("cosh(1)"), 1.5430806348152437, 1e-9));
    }

    #[test]
    fn tanh_pos() {
        assert!(approx(ev("tanh(1)"), 0.7615941559557649, 1e-9));
    }

    #[test]
    fn hyperbolic_identities() {
        // cosh^2 - sinh^2 = 1
        let x = 1.5_f64;
        let c = ev(&format!("cosh({})", x));
        let s = ev(&format!("sinh({})", x));
        assert!(approx(c * c - s * s, 1.0, 1e-9));
    }

    #[test]
    fn mod_large() {
        assert_eq!(ev("mod(100,7)"), 2.0);
    }

    #[test]
    fn npr_identity() {
        // nPr 1 = n
        for n in [1.0, 2.0, 3.0, 5.0, 10.0] {
            assert_eq!(ev(&format!("nPr({},1)", n)), n);
        }
    }

    #[test]
    fn ncr_identity() {
        // nCr n = 1
        for n in [1.0, 5.0, 10.0, 20.0] {
            assert_eq!(ev(&format!("nCr({},{})", n, n)), 1.0);
        }
    }

    #[test]
    fn ncr_pascal() {
        // nCr(5,2) + nCr(5,3) = nCr(6,3)
        assert!(approx(ev("nCr(5,2)+nCr(5,3)"), ev("nCr(6,3)"), 1e-9));
    }

    #[test]
    fn random_bounded() {
        for _ in 0..200 {
            let r = ev("random()");
            assert!((0.0..1.0).contains(&r));
            assert!(r.is_finite());
        }
    }

    #[test]
    fn factorial_in_sum() {
        assert_eq!(ev("3!+4!"), 6.0 + 24.0);
    }

    #[test]
    fn power_in_sum() {
        assert_eq!(ev("2^3+3^2"), 8.0 + 9.0);
    }

    #[test]
    fn trig_in_power() {
        // sin(pi/2) = 1, then 1^5 = 1
        assert!(approx(ev("sin(pi/2)^5"), 1.0, 1e-9));
    }

    #[test]
    fn power_in_trig() {
        // sin(pi^0) = sin(1)
        assert!(approx(ev("sin(pi^0)"), 1.0_f64.sin(), 1e-9));
    }

    #[test]
    fn abs_of_expr() {
        assert!(approx(ev("abs(1-100)"), 99.0, 1e-9));
    }

    #[test]
    fn max_of_three_via_chain() {
        assert!(approx(ev("max(max(1,2),3)"), 3.0, 1e-9));
    }

    #[test]
    fn min_of_three_via_chain() {
        assert!(approx(ev("min(min(5,3),1)"), 1.0, 1e-9));
    }

    #[test]
    fn nested_unary_paren() {
        assert_eq!(ev("-(-(-3))"), -3.0);
    }

    #[test]
    fn triple_unary() {
        assert_eq!(ev("---3"), -3.0);
    }

    #[test]
    fn quad_unary() {
        assert_eq!(ev("----3"), 3.0);
    }

    #[test]
    fn leading_spaces_unary() {
        assert_eq!(ev("  -  5  "), -5.0);
    }

    #[test]
    fn e_in_exp() {
        assert!(approx(ev("exp(1)"), E, 1e-9));
    }

    #[test]
    fn two_pi() {
        assert!(approx(ev("2*pi"), 2.0 * PI, 1e-9));
    }

    #[test]
    fn pi_divided_by_e() {
        assert!(approx(ev("pi/e"), PI / E, 1e-9));
    }

    #[test]
    fn log10_powers_of_ten() {
        for n in 0..15 {
            let s = format!("log(10^{})", n);
            assert!(approx(ev(&s), n as f64, 1e-9));
        }
    }

    #[test]
    fn ln_of_e() {
        assert!(approx(ev("ln(e)"), 1.0, 1e-9));
    }

    #[test]
    fn multiple_spaces() {
        assert_eq!(ev("1    +    2"), 3.0);
    }

    #[test]
    fn tab_separated() {
        assert_eq!(ev("1\t+\t2"), 3.0);
    }

    #[test]
    fn newline_separated() {
        assert_eq!(ev("1\n+\n2"), 3.0);
    }

    #[test]
    fn no_space_around_ops() {
        assert_eq!(ev("1+2*3"), 7.0);
    }

    #[test]
    fn long_number_chain() {
        assert_eq!(ev("1+2+3+4+5+6+7+8+9+10+11+12+13+14+15"), 120.0);
    }

    #[test]
    fn product_chain() {
        assert_eq!(ev("1*2*3*4*5*6*7*8*9*10"), 3_628_800.0);
    }

    #[test]
    fn pow_chain_left() {
        // 2^2^2^2 = 2^(2^(2^2)) = 2^16 = 65536 (right-assoc)
        assert_eq!(ev("2^2^2^2"), 65536.0);
    }

    #[test]
    fn with_var_doesnt_exist() {
        assert!(err("x+1").contains("Unknown"));
    }

    #[test]
    fn pi_2() {
        assert!(approx(ev("pi/2"), PI / 2.0, 1e-9));
    }

    #[test]
    fn large_arithmetic() {
        assert_eq!(ev("999999+1"), 1_000_000.0);
    }

    #[test]
    fn very_small() {
        let v = 1e-300_f64;
        let s = format!("{}", v);
        let r = ev(&s);
        assert!(approx(r, v, 1e-310));
    }

    #[test]
    fn very_large() {
        let v = 1e100_f64;
        let s = format!("{}", v);
        let r = ev(&s);
        assert!(approx(r, v, 1e90));
    }

    #[test]
    fn exp_in_ln() {
        assert!(approx(ev("ln(exp(2)+0)"), 2.0, 1e-9));
    }

    #[test]
    fn sqrts_sum() {
        let v = ev("sqrt(9)+sqrt(16)+sqrt(25)");
        assert!(approx(v, 3.0 + 4.0 + 5.0, 1e-9));
    }

    #[test]
    fn abs_then_floor() {
        assert!(approx(ev("floor(abs(-3.7))"), 3.0, 1e-9));
    }

    #[test]
    fn sin_of_pi_over_6() {
        assert!(approx(evd("sin(30)"), 0.5, 1e-9));
    }

    #[test]
    fn cos_of_pi_over_3() {
        assert!(approx(evd("cos(60)"), 0.5, 1e-9));
    }

    #[test]
    fn tan_of_pi_over_4() {
        assert!(approx(evd("tan(45)"), 1.0, 1e-9));
    }

    #[test]
    fn sin_of_0() {
        assert_eq!(evd("sin(0)"), 0.0);
    }

    #[test]
    fn cos_of_0() {
        assert_eq!(evd("cos(0)"), 1.0);
    }

    #[test]
    fn cos_of_90() {
        assert!(approx(evd("cos(90)"), 0.0, 1e-9));
    }

    #[test]
    fn sin_of_180() {
        assert!(approx(evd("sin(180)"), 0.0, 1e-9));
    }

    #[test]
    fn cos_of_180() {
        assert!(approx(evd("cos(180)"), -1.0, 1e-9));
    }

    #[test]
    fn sin_of_360() {
        assert!(approx(evd("sin(360)"), 0.0, 1e-9));
    }

    #[test]
    fn neg_atan() {
        assert!(approx(evd("atan(-1)"), -45.0, 1e-9));
    }

    #[test]
    fn asin_neg() {
        assert!(approx(evd("asin(-0.5)"), -30.0, 1e-9));
    }

    #[test]
    fn acos_neg() {
        assert!(approx(evd("acos(-0.5)"), 120.0, 1e-9));
    }

    #[test]
    fn format_neg_int() {
        assert_eq!(format_result(-42.0), "-42");
    }

    #[test]
    fn format_pos_dec() {
        assert_eq!(format_result(3.14), "3.14");
    }

    #[test]
    fn format_pos_small() {
        let s = format_result(0.0001);
        assert!(s == "0.0001" || s == "1e-4" || s.contains("e"));
    }

    #[test]
    fn format_pos_huge() {
        let s = format_result(1e30);
        assert!(s.contains("e"));
    }

    #[test]
    fn format_just_below_eps() {
        let s = format_result(1e-11);
        assert!(s == "0" || s.contains("e"));
    }

    #[test]
    fn format_inf() {
        assert_eq!(format_result(f64::INFINITY), "Error");
    }

    #[test]
    fn format_neg_inf() {
        assert_eq!(format_result(f64::NEG_INFINITY), "Error");
    }

    #[test]
    fn try_eval_partial() {
        assert_eq!(try_evaluate_internal("1+", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("sin(", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("(1+2", AngleMode::Rad), None);
        assert_eq!(try_evaluate_internal("1/0", AngleMode::Rad), None);
    }

    #[test]
    fn try_eval_valid() {
        assert_eq!(try_evaluate_internal("1+1", AngleMode::Rad), Some(2.0));
        assert_eq!(try_evaluate_internal("pi", AngleMode::Rad), Some(PI));
    }

    #[test]
    fn format_nan_string() {
        assert_eq!(format_result(f64::NAN), "Error");
    }

    #[test]
    fn large_pow() {
        assert!(approx(ev("10^5"), 100_000.0, 1e-6));
        assert!(approx(ev("10^6"), 1_000_000.0, 1.0));
    }

    #[test]
    fn small_pow() {
        assert!(approx(ev("10^-2"), 0.01, 1e-9));
        assert!(approx(ev("10^-3"), 0.001, 1e-9));
    }

    #[test]
    fn negative_to_fractional() {
        // (-8)^(1/3) returns Math error; use cbrt for real cube root
        assert!(approx(ev("cbrt(-8)"), -2.0, 1e-9));
    }

    #[test]
    fn real_world_1() {
        // 5-year compound interest: P(1+r/n)^(nt)
        let p: f64 = 1000.0;
        let r: f64 = 0.05;
        let n: f64 = 12.0;
        let t: f64 = 5.0;
        let s = format!("{}*(1+{}/{})^({}*{})", p, r, n, n, t);
        let v = ev(&s);
        let expected = p * (1.0 + r / n).powf(n * t);
        assert!(approx(v, expected, 1e-6));
    }

    #[test]
    fn real_world_2() {
        // Circle area: pi*r^2
        let r = 5.0;
        let v = ev(&format!("pi*{}^2", r));
        assert!(approx(v, PI * r * r, 1e-9));
    }

    #[test]
    fn real_world_3() {
        // Pythagorean: sqrt(a^2 + b^2) = c
        let (a, b) = (3.0, 4.0);
        let v = ev(&format!("sqrt({}^2+{}^2)", a, b));
        assert!(approx(v, 5.0, 1e-9));
    }

    #[test]
    fn real_world_4() {
        // Binet's formula: F_n = (phi^n - (-phi)^-n) / sqrt(5)
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        for n in 1..=10 {
            let s = format!("(({}^{})-(-{})^(-{}))/sqrt(5)", phi, n, phi, n);
            let v = ev(&s);
            // Just check it's finite and positive for n>=1
            assert!(v.is_finite());
            assert!(v > 0.0);
        }
    }

    #[test]
    fn complex_nested_call() {
        // sqrt(16)=4, pow(2,3)=8, min(8,7)=7, max(4,7)=7
        assert!(approx(
            ev("max(sqrt(16), min(pow(2,3), 7))"),
            7.0,
            1e-9
        ));
    }

    #[test]
    fn function_in_function() {
        assert!(approx(ev("sin(asin(0.5))"), 0.5, 1e-9));
    }

    #[test]
    fn deep_call_stack() {
        assert!(approx(
            ev("abs(floor(ceil(round(ln(exp(1))))))"),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn numbers_in_functions() {
        assert!(approx(ev("log(1000)"), 3.0, 1e-9));
        assert!(approx(ev("ln(exp(3))"), 3.0, 1e-9));
        assert!(approx(ev("sqrt(sqrt(256))"), 4.0, 1e-9));
    }

    #[test]
    fn divide_by_self() {
        for n in [1.0, 2.0, 5.0, 10.0, 100.0] {
            assert!(approx(ev(&format!("{}/{}", n, n)), 1.0, 1e-9));
        }
    }

    #[test]
    fn multiply_by_zero() {
        for n in [0.0, 1.0, 100.0, -5.0] {
            assert_eq!(ev(&format!("{}*0", n)), 0.0);
        }
    }

    #[test]
    fn add_zero() {
        for n in [0.0, 1.0, -5.0, 100.0] {
            assert_eq!(ev(&format!("{}+0", n)), n);
        }
    }

    #[test]
    fn sub_zero() {
        for n in [0.0, 1.0, -5.0, 100.0] {
            assert_eq!(ev(&format!("{}-0", n)), n);
        }
    }

    #[test]
    fn sub_self() {
        for n in [0.0, 1.0, -5.0, 100.0] {
            assert_eq!(ev(&format!("{}-{}", n, n)), 0.0);
        }
    }

    #[test]
    fn pow_zero_exp() {
        for base in [2.0, 3.0, 10.0, -5.0] {
            assert_eq!(ev(&format!("{}^0", base)), 1.0);
        }
    }

    #[test]
    fn one_pow_n() {
        for n in 0..20 {
            let s = format!("1^{}", n);
            assert_eq!(ev(&s), 1.0);
        }
    }

    #[test]
    fn neg_n_factorial() {
        // Can't do (-1)! in real math, but we can do 1 - n!
        assert_eq!(ev("1-3!"), -5.0);
    }

    #[test]
    fn format_trailing_zero() {
        // 0.5 should not display as "0.5000..."
        let s = format_result(0.5);
        assert!(!s.contains("0000"));
    }

    #[test]
    fn format_neg_zero() {
        assert_eq!(format_result(-0.0), "0");
    }

    #[test]
    fn single_paren_group() {
        assert_eq!(ev("(42)"), 42.0);
    }

    #[test]
    fn parens_in_arg() {
        assert_eq!(ev("min((1+2),(3+4))"), 3.0);
    }

    #[test]
    fn nested_pow_pow() {
        assert_eq!(ev("((2^3)^2)"), 64.0);
    }

    #[test]
    fn explicit_pos_sign() {
        assert_eq!(ev("+5"), 5.0);
        assert_eq!(ev("+5+5"), 10.0);
    }

    #[test]
    fn many_unary() {
        assert_eq!(ev("----5"), 5.0);
        assert_eq!(ev("---5"), -5.0);
        assert_eq!(ev("---(-5)"), 5.0);
    }

    #[test]
    fn big_eng_combo() {
        // 1M + 500k = 1.5M
        assert_eq!(ev("1M+500k"), 1_500_000.0);
    }

    #[test]
    fn subtract_eng() {
        assert_eq!(ev("2M-500k"), 1_500_000.0);
    }

    #[test]
    fn multiply_eng() {
        assert_eq!(ev("3*2k"), 6_000.0);
    }

    #[test]
    fn eng_then_pow() {
        assert_eq!(ev("1k^2"), 1_000_000.0);
    }

    #[test]
    fn div_by_eng() {
        assert_eq!(ev("1M/1k"), 1_000.0);
    }

    #[test]
    fn degree_to_radian_formula() {
        // angle_rad = angle_deg * pi / 180
        let d = 45.0;
        let v = ev(&format!("{}*pi/180", d));
        assert!(approx(v, d * PI / 180.0, 1e-9));
    }

    #[test]
    fn radian_to_degree_formula() {
        let r = PI / 4.0;
        let v = ev(&format!("{}*180/pi", r));
        assert!(approx(v, 45.0, 1e-9));
    }

    #[test]
    fn factorial_1() {
        assert_eq!(ev("1!"), 1.0);
    }

    #[test]
    fn factorial_5() {
        assert_eq!(ev("5!"), 120.0);
    }

    #[test]
    fn factorial_6() {
        assert_eq!(ev("6!"), 720.0);
    }

    #[test]
    fn factorial_7() {
        assert_eq!(ev("7!"), 5040.0);
    }

    #[test]
    fn factorial_8() {
        assert_eq!(ev("8!"), 40320.0);
    }

    #[test]
    fn factorial_9() {
        assert_eq!(ev("9!"), 362880.0);
    }

    #[test]
    fn factorial_10() {
        assert_eq!(ev("10!"), 3_628_800.0);
    }

    #[test]
    fn pi_in_degrees() {
        // 180 degrees = pi radians
        let v = evaluate_internal("180*pi/180", AngleMode::Rad).unwrap();
        assert!(approx(v, PI, 1e-9));
    }

    #[test]
    fn random_seed_stable_native() {
        // Native LCG should produce stable values within bounds
        for _ in 0..100 {
            let r = ev("random()");
            assert!((0.0..1.0).contains(&r));
        }
    }

    #[test]
    fn chained_eng() {
        // 1k + 1k = 2k (no fold)
        assert_eq!(ev("1k+1k"), 2_000.0);
    }

    #[test]
    fn ops_with_spaces() {
        assert_eq!(ev("  1  +  2  *  3  "), 7.0);
    }

    #[test]
    fn all_ops() {
        assert!(approx(
            ev("1+2*3-4/5%3+2^3-10"),
            1.0 + 6.0 - 0.8 + 8.0 - 10.0,
            1e-9
        ));
    }

    #[test]
    fn giant_factorial_in_expr() {
        assert_eq!(ev("5!+4!+3!+2!+1!+0!"), (120 + 24 + 6 + 2 + 1 + 1) as f64);
    }

    #[test]
    fn sin_pi_over_4_times_sqrt2() {
        // sin(pi/4) = sqrt(2)/2
        let s = ev("sin(pi/4)");
        let r = ev("sqrt(2)/2");
        assert!(approx(s, r, 1e-9));
    }

    #[test]
    fn two_to_one_zero() {
        assert!(approx(ev("2^10"), 1024.0, 1e-9));
    }

    #[test]
    fn ten_to_three() {
        assert!(approx(ev("10^3"), 1000.0, 1e-9));
    }

    #[test]
    fn log1() {
        assert!(approx(ev("log(1)"), 0.0, 1e-9));
    }

    #[test]
    fn ln1() {
        assert!(approx(ev("ln(1)"), 0.0, 1e-9));
    }

    #[test]
    fn exp0() {
        assert!(approx(ev("exp(0)"), 1.0, 1e-9));
    }

    #[test]
    fn pow0() {
        assert!(approx(ev("pow(2,0)"), 1.0, 1e-9));
        assert!(approx(ev("pow(0,5)"), 0.0, 1e-9));
        assert!(approx(ev("pow(0,0)"), 1.0, 1e-9));
    }

    #[test]
    fn chained_factorial_2() {
        // (3!)! = 6! = 720
        assert_eq!(ev("(3!)!"), 720.0);
    }

    #[test]
    fn comma_only_arg() {
        assert_eq!(ev("min(5,3)"), 3.0);
    }
}

#[wasm_bindgen(start)]
pub fn _start() {}
