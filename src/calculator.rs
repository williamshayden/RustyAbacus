/// Expression evaluator with scientific function support
/// Supports: +, -, *, /, ^, %, sin, cos, tan, asin, acos, atan, log, ln, sqrt, abs, floor, ceil, factorial
/// Variables: x, t (for graphing)

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Token {
    Number(f64),
    Variable(char),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Modulo,
    Factorial,
    LParen,
    RParen,
    // Scientific functions
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Log,
    Ln,
    Sqrt,
    Abs,
    Floor,
    Ceil,
}

/// Evaluate an expression without variables
pub fn evaluate(expr: &str) -> Result<f64, String> {
    evaluate_with_vars(expr, &HashMap::new())
}

/// Evaluate an expression with a single variable
pub fn evaluate_with_var(expr: &str, var: char, value: f64) -> Result<f64, String> {
    let mut vars = HashMap::new();
    vars.insert(var, value);
    evaluate_with_vars(expr, &vars)
}

/// Evaluate an expression with multiple variables
pub fn evaluate_with_vars(expr: &str, vars: &HashMap<char, f64>) -> Result<f64, String> {
    if expr.trim().is_empty() {
        return Err("Empty expression".to_string());
    }
    let tokens = tokenize(expr)?;
    let rpn = shunting_yard(tokens)?;
    evaluate_rpn(rpn, vars)
}

/// Check if an expression contains a specific variable
pub fn contains_variable(expr: &str, var: char) -> bool {
    expr.to_lowercase().contains(var)
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    let mut buffer = String::new();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                // Flush any alpha buffer as a function name or variable
                if !buffer.is_empty() {
                    tokens.push(parse_identifier(&buffer)?);
                    buffer.clear();
                }
                buffer.push(c);
                chars.next();
                // Continue collecting digits
                while let Some(&nc) = chars.peek() {
                    if nc.is_ascii_digit() || nc == '.' {
                        buffer.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let num = buffer
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid number: {}", buffer))?;
                tokens.push(Token::Number(num));
                buffer.clear();
            }
            'a'..='z' | 'A'..='Z' => {
                buffer.push(c.to_ascii_lowercase());
                chars.next();
            }
            '+' | '-' | '*' | '/' | '^' | '%' | '!' | '(' | ')' => {
                // Flush buffer if it contains a function name or variable
                if !buffer.is_empty() {
                    tokens.push(parse_identifier(&buffer)?);
                    buffer.clear();
                }

                let token = match c {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '^' => Token::Power,
                    '%' => Token::Modulo,
                    '!' => Token::Factorial,
                    '(' => Token::LParen,
                    ')' => Token::RParen,
                    _ => unreachable!(),
                };
                tokens.push(token);
                chars.next();
            }
            _ => return Err(format!("Invalid character: {}", c)),
        }
    }

    // Flush remaining buffer
    if !buffer.is_empty() {
        if buffer.chars().all(|c| c.is_ascii_digit() || c == '.') {
            let num = buffer
                .parse::<f64>()
                .map_err(|_| format!("Invalid number: {}", buffer))?;
            tokens.push(Token::Number(num));
        } else {
            tokens.push(parse_identifier(&buffer)?);
        }
    }

    Ok(tokens)
}

fn parse_identifier(name: &str) -> Result<Token, String> {
    match name {
        // Scientific functions
        "sin" => Ok(Token::Sin),
        "cos" => Ok(Token::Cos),
        "tan" => Ok(Token::Tan),
        "asin" | "arcsin" => Ok(Token::Asin),
        "acos" | "arccos" => Ok(Token::Acos),
        "atan" | "arctan" => Ok(Token::Atan),
        "log" => Ok(Token::Log),
        "ln" => Ok(Token::Ln),
        "sqrt" => Ok(Token::Sqrt),
        "abs" => Ok(Token::Abs),
        "floor" => Ok(Token::Floor),
        "ceil" => Ok(Token::Ceil),
        // Variables
        "x" => Ok(Token::Variable('x')),
        "t" => Ok(Token::Variable('t')),
        // Mathematical constants
        "pi" => Ok(Token::Number(std::f64::consts::PI)),
        "e" => Ok(Token::Number(std::f64::consts::E)),
        // Special variable for previous answer
        "ans" => Ok(Token::Variable('a')),
        _ => Err(format!("Unknown identifier: {}", name)),
    }
}

fn is_function(token: &Token) -> bool {
    matches!(
        token,
        Token::Sin | Token::Cos | Token::Tan | Token::Asin | Token::Acos | Token::Atan |
        Token::Log | Token::Ln | Token::Sqrt | Token::Abs | Token::Floor | Token::Ceil
    )
}

fn precedence(token: &Token) -> u8 {
    match token {
        Token::Plus | Token::Minus => 1,
        Token::Multiply | Token::Divide | Token::Modulo => 2,
        Token::Power => 3,
        Token::Factorial => 5, // Highest precedence (postfix)
        Token::Sin | Token::Cos | Token::Tan | Token::Asin | Token::Acos | Token::Atan |
        Token::Log | Token::Ln | Token::Sqrt | Token::Abs | Token::Floor | Token::Ceil => 4,
        _ => 0,
    }
}

fn is_left_associative(token: &Token) -> bool {
    !matches!(token, Token::Power)
}

fn shunting_yard(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut output = Vec::new();
    let mut operators: Vec<Token> = Vec::new();

    for token in tokens {
        match &token {
            Token::Number(_) | Token::Variable(_) => output.push(token),
            Token::Sin | Token::Cos | Token::Tan | Token::Asin | Token::Acos | Token::Atan |
            Token::Log | Token::Ln | Token::Sqrt | Token::Abs | Token::Floor | Token::Ceil => {
                operators.push(token);
            }
            Token::Factorial => {
                // Factorial is a postfix operator - immediately apply to the last number
                output.push(token);
            }
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power | Token::Modulo => {
                while let Some(top) = operators.last() {
                    if matches!(top, Token::LParen) {
                        break;
                    }
                    if is_function(top) {
                        output.push(operators.pop().unwrap());
                        continue;
                    }
                    if precedence(top) > precedence(&token)
                        || (precedence(top) == precedence(&token) && is_left_associative(&token))
                    {
                        output.push(operators.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operators.push(token);
            }
            Token::LParen => operators.push(token),
            Token::RParen => {
                let mut found_lparen = false;
                while let Some(op) = operators.pop() {
                    if matches!(op, Token::LParen) {
                        found_lparen = true;
                        break;
                    }
                    output.push(op);
                }
                if !found_lparen {
                    return Err("Mismatched parentheses".to_string());
                }
                // If there's a function on top of the stack, pop it
                if let Some(top) = operators.last() {
                    if is_function(top) {
                        output.push(operators.pop().unwrap());
                    }
                }
            }
        }
    }

    while let Some(op) = operators.pop() {
        if matches!(op, Token::LParen | Token::RParen) {
            return Err("Mismatched parentheses".to_string());
        }
        output.push(op);
    }

    Ok(output)
}

fn evaluate_rpn(tokens: Vec<Token>, vars: &HashMap<char, f64>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => stack.push(n),
            Token::Variable(v) => {
                let value = vars
                    .get(&v)
                    .ok_or_else(|| format!("Undefined variable: {}", v))?;
                stack.push(*value);
            }
            Token::Plus => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a + b);
            }
            Token::Minus => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a - b);
            }
            Token::Multiply => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a * b);
            }
            Token::Divide => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                stack.push(a / b);
            }
            Token::Power => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.powf(b));
            }
            Token::Modulo => {
                let b = stack.pop().ok_or("Invalid expression")?;
                let a = stack.pop().ok_or("Invalid expression")?;
                if b == 0.0 {
                    return Err("Modulo by zero".to_string());
                }
                stack.push(a % b);
            }
            Token::Factorial => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a < 0.0 || a.fract() != 0.0 {
                    return Err("Factorial requires non-negative integer".to_string());
                }
                stack.push(factorial(a as u64));
            }
            // Scientific functions (unary)
            Token::Sin => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.sin());
            }
            Token::Cos => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.cos());
            }
            Token::Tan => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.tan());
            }
            Token::Asin => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a < -1.0 || a > 1.0 {
                    return Err("asin domain error".to_string());
                }
                stack.push(a.asin());
            }
            Token::Acos => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a < -1.0 || a > 1.0 {
                    return Err("acos domain error".to_string());
                }
                stack.push(a.acos());
            }
            Token::Atan => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.atan());
            }
            Token::Log => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a <= 0.0 {
                    return Err("log of non-positive number".to_string());
                }
                stack.push(a.log10());
            }
            Token::Ln => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a <= 0.0 {
                    return Err("ln of non-positive number".to_string());
                }
                stack.push(a.ln());
            }
            Token::Sqrt => {
                let a = stack.pop().ok_or("Invalid expression")?;
                if a < 0.0 {
                    return Err("sqrt of negative number".to_string());
                }
                stack.push(a.sqrt());
            }
            Token::Abs => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.abs());
            }
            Token::Floor => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.floor());
            }
            Token::Ceil => {
                let a = stack.pop().ok_or("Invalid expression")?;
                stack.push(a.ceil());
            }
            Token::LParen | Token::RParen => {
                return Err("Invalid token in RPN".to_string());
            }
        }
    }

    if stack.len() != 1 {
        return Err("Invalid expression".to_string());
    }

    Ok(stack[0])
}

/// Calculate factorial of a non-negative integer
fn factorial(n: u64) -> f64 {
    if n <= 1 {
        1.0
    } else if n > 170 {
        f64::INFINITY // Overflow for large factorials
    } else {
        (2..=n).fold(1.0, |acc, x| acc * x as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert!((evaluate("2 + 3").unwrap() - 5.0).abs() < 1e-10);
        assert!((evaluate("10 - 4").unwrap() - 6.0).abs() < 1e-10);
        assert!((evaluate("3 * 4").unwrap() - 12.0).abs() < 1e-10);
        assert!((evaluate("15 / 3").unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_power() {
        assert!((evaluate("2^8").unwrap() - 256.0).abs() < 1e-10);
        assert!((evaluate("3^2").unwrap() - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_scientific_functions() {
        assert!((evaluate("sin(0)").unwrap() - 0.0).abs() < 1e-10);
        assert!((evaluate("cos(0)").unwrap() - 1.0).abs() < 1e-10);
        assert!((evaluate("sqrt(16)").unwrap() - 4.0).abs() < 1e-10);
        assert!((evaluate("log(100)").unwrap() - 2.0).abs() < 1e-10);
        assert!((evaluate("ln(1)").unwrap() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_complex_expressions() {
        assert!((evaluate("2 + 3 * 4").unwrap() - 14.0).abs() < 1e-10);
        assert!((evaluate("(2 + 3) * 4").unwrap() - 20.0).abs() < 1e-10);
        assert!((evaluate("sqrt(16) + 2^3").unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_variables() {
        assert!((evaluate_with_var("x", 'x', 5.0).unwrap() - 5.0).abs() < 1e-10);
        assert!((evaluate_with_var("x^2", 'x', 3.0).unwrap() - 9.0).abs() < 1e-10);
        assert!((evaluate_with_var("sin(x)", 'x', 0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((evaluate_with_var("2*x + 1", 'x', 4.0).unwrap() - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_constants() {
        assert!((evaluate("pi").unwrap() - std::f64::consts::PI).abs() < 1e-10);
        assert!((evaluate("e").unwrap() - std::f64::consts::E).abs() < 1e-10);
        assert!((evaluate("2*pi").unwrap() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_trig() {
        assert!((evaluate("asin(0)").unwrap() - 0.0).abs() < 1e-10);
        assert!((evaluate("acos(1)").unwrap() - 0.0).abs() < 1e-10);
        assert!((evaluate("atan(0)").unwrap() - 0.0).abs() < 1e-10);
        assert!((evaluate("asin(1)").unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_abs_floor_ceil() {
        assert!((evaluate("abs(0-5)").unwrap() - 5.0).abs() < 1e-10);
        assert!((evaluate("abs(5)").unwrap() - 5.0).abs() < 1e-10);
        assert!((evaluate("floor(3.7)").unwrap() - 3.0).abs() < 1e-10);
        assert!((evaluate("ceil(3.2)").unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_factorial() {
        assert!((evaluate("5!").unwrap() - 120.0).abs() < 1e-10);
        assert!((evaluate("0!").unwrap() - 1.0).abs() < 1e-10);
        assert!((evaluate("1!").unwrap() - 1.0).abs() < 1e-10);
        assert!((evaluate("3! + 2").unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_modulo() {
        assert!((evaluate("10 % 3").unwrap() - 1.0).abs() < 1e-10);
        assert!((evaluate("15 % 4").unwrap() - 3.0).abs() < 1e-10);
        assert!((evaluate("8 % 2").unwrap() - 0.0).abs() < 1e-10);
    }
}
