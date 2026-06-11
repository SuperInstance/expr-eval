//! expr-eval — Evaluate arithmetic expression ASTs with environment variables.

use std::collections::HashMap;

/// Supported binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BinOp { Add, Sub, Mul, Div, Pow }

/// AST for arithmetic expressions with variable lookup.
#[derive(Debug, Clone)]
enum Expr {
    Literal(f64),
    Var(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    UnaryNeg(Box<Expr>),
    Call { name: String, args: Vec<Expr> },
}

/// Evaluation environment (variable bindings + function table).
struct Env {
    vars: HashMap<String, f64>,
    funcs: HashMap<String, fn(Vec<f64>) -> f64>,
}

impl Env {
    fn new() -> Self {
        let mut vars = HashMap::new();
        vars.insert("pi".into(), std::f64::consts::PI);
        vars.insert("e".into(), std::f64::consts::E);

        let mut funcs = HashMap::new();
        fn f_sqrt(args: Vec<f64>) -> f64 { args[0].sqrt() }
        fn f_abs(args: Vec<f64>) -> f64 { args[0].abs() }
        fn f_sin(args: Vec<f64>) -> f64 { args[0].sin() }
        fn f_cos(args: Vec<f64>) -> f64 { args[0].cos() }
        fn f_max(args: Vec<f64>) -> f64 { args.iter().cloned().fold(f64::NEG_INFINITY, f64::max) }
        fn f_min(args: Vec<f64>) -> f64 { args.iter().cloned().fold(f64::INFINITY, f64::min) }

        type FnSig = fn(Vec<f64>) -> f64;
        funcs.insert("sqrt".into(), f_sqrt as FnSig);
        funcs.insert("abs".into(), f_abs as FnSig);
        funcs.insert("sin".into(), f_sin as FnSig);
        funcs.insert("cos".into(), f_cos as FnSig);
        funcs.insert("max".into(), f_max as FnSig);
        funcs.insert("min".into(), f_min as FnSig);

        Self { vars, funcs }
    }

    fn with(mut self, name: &str, val: f64) -> Self {
        self.vars.insert(name.to_string(), val);
        self
    }
}

/// Evaluation error.
#[derive(Debug)]
enum EvalError {
    UndefinedVar(String),
    UndefinedFunc(String),
    Arity { name: String, expected: usize, got: usize },
    DivisionByZero,
}

/// Evaluate an expression against an environment.
fn eval(expr: &Expr, env: &Env) -> Result<f64, EvalError> {
    match expr {
        Expr::Literal(n) => Ok(*n),
        Expr::Var(name) => env.vars.get(name).copied().ok_or_else(|| EvalError::UndefinedVar(name.clone())),
        Expr::UnaryNeg(inner) => Ok(-eval(inner, env)?),
        Expr::Binary { op, left, right } => {
            let l = eval(left, env)?;
            let r = eval(right, env)?;
            match op {
                BinOp::Add => Ok(l + r),
                BinOp::Sub => Ok(l - r),
                BinOp::Mul => Ok(l * r),
                BinOp::Div => if r == 0.0 { Err(EvalError::DivisionByZero) } else { Ok(l / r) },
                BinOp::Pow => Ok(l.powf(r)),
            }
        }
        Expr::Call { name, args } => {
            let func = env.funcs.get(name).ok_or_else(|| EvalError::UndefinedFunc(name.clone()))?;
            let vals: Vec<f64> = args.iter().map(|a| eval(a, env)).collect::<Result<_, _>>()?;
            Ok(func(vals))
        }
    }
}

/// Format result nicely.
fn fmt_result(r: Result<f64, EvalError>) -> String {
    match r {
        Ok(v) => format!("{v:.6}"),
        Err(EvalError::UndefinedVar(v)) => format!("ERR: undefined var '{v}'"),
        Err(EvalError::UndefinedFunc(f)) => format!("ERR: undefined func '{f}'"),
        Err(EvalError::Arity { name, expected, got }) => format!("ERR: {name}() expects {expected} args, got {got}"),
        Err(EvalError::DivisionByZero) => "ERR: division by zero".into(),
    }
}

fn main() {
    let env = Env::new()
        .with("x", 3.0)
        .with("y", 7.0);

    let tests: Vec<(Expr, &str)> = vec![
        (Expr::Binary { op: BinOp::Add, left: Box::new(Expr::Literal(2.0)), right: Box::new(Expr::Literal(3.0)) }, "2 + 3"),
        (Expr::Binary { op: BinOp::Mul, left: Box::new(Expr::Var("x".into())), right: Box::new(Expr::Var("y".into())) }, "x * y"),
        (Expr::Call { name: "sqrt".into(), args: vec![Expr::Binary { op: BinOp::Add, left: Box::new(Expr::Var("x".into())), right: Box::new(Expr::Literal(1.0)) }] }, "sqrt(x + 1)"),
        (Expr::Binary { op: BinOp::Pow, left: Box::new(Expr::Var("x".into())), right: Box::new(Expr::Literal(3.0)) }, "x^3"),
        (Expr::Binary { op: BinOp::Div, left: Box::new(Expr::Literal(1.0)), right: Box::new(Expr::Literal(0.0)) }, "1/0"),
        (Expr::Call { name: "sin".into(), args: vec![Expr::Var("pi".into())] }, "sin(pi)"),
    ];

    println!("expr-eval — expression evaluator");
    println!("===============================\n");
    for (expr, desc) in &tests {
        let result = eval(expr, &env);
        println!("{desc:20} => {}", fmt_result(result));
    }
}
