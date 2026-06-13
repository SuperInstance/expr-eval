# Expression Evaluator

**expr-eval** is a tree-walking expression evaluator for Rust that evaluates arithmetic ASTs with variable lookup, built-in functions, and explicit error handling. It supports `sqrt`, `sin`, `cos`, `abs`, `max`, `min`, the power operator, and user-supplied variable bindings.

## Why It Matters

Expression evaluation is a core building block for configuration systems, spreadsheet engines, database query planners, and interactive calculators. Unlike a bytecode VM (which compiles then executes), a tree-walking evaluator evaluates the AST directly — no compilation step, no state to manage. This makes it ideal for situations where expressions are evaluated once or infrequently: rule engines, template systems, and one-shot computations. The explicit `EvalError` enum ensures that division by zero, undefined variables, and unknown functions produce structured errors rather than panics.

## How It Works

### Evaluation Model

The evaluator recursively walks the AST. Each node type has a defined evaluation rule:

```
eval(Literal(n))       = n
eval(Var(name))        = env.vars[name]        // or UndefinedVar error
eval(UnaryNeg(inner))  = -eval(inner)
eval(Binary(op, l, r)) = op(eval(l), eval(r))  // DivisionByZero if r=0
eval(Call(name, args)) = env.funcs[name](args)  // or UndefinedFunc error
```

**Complexity:** O(N) per evaluation where N = AST nodes. Each node is visited exactly once. Space complexity is O(D) where D = AST depth (call stack).

### Environment

The `Env` struct holds:
- **`vars: HashMap<String, f64>`** — Variable bindings. Pre-populated with `pi = π` and `e = Euler's number`.
- **`funcs: HashMap<String, fn(Vec<f64>) -> f64>`** — Built-in functions as function pointers.

Users extend the environment via `.with("x", 3.0)` for custom variables.

### Operator Semantics

| Operator | Behavior |
|----------|----------|
| `Add` | Standard addition |
| `Sub` | Standard subtraction |
| `Mul` | Standard multiplication |
| `Div` | Returns `DivisionByZero` error if divisor is exactly 0.0 |
| `Pow` | `l.powf(r)` — standard IEEE 754 power |

### Error Model

```rust
enum EvalError {
    UndefinedVar(String),
    UndefinedFunc(String),
    Arity { name: String, expected: usize, got: usize },
    DivisionByZero,
}
```

Errors short-circuit: if evaluating the left subtree of a binary node fails, the right subtree is never evaluated (Rust's `?` operator propagates immediately).

### Comparison to Bytecode VM

Tree-walking: O(N) per evaluation, O(0) compilation. Better for one-shot evaluations.
Bytecode VM: O(B) per evaluation, O(N) compilation. Better for repeated evaluations of the same expression.

## Quick Start

```rust
// This is a binary crate. Run with: cargo run

use expr_eval::{eval, Env, Expr, BinOp};

fn main() {
    let env = Env::new()
        .with("x", 3.0)
        .with("y", 7.0);

    // Evaluate: sqrt(x + 1)
    let expr = Expr::Call {
        name: "sqrt".into(),
        args: vec![Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Expr::Var("x".into())),
            right: Box::new(Expr::Literal(1.0)),
        }],
    };

    match eval(&expr, &env) {
        Ok(result) => println!("Result: {:.6}", result), // Result: 2.000000
        Err(e) => println!("Error: {:?}", e),
    }

    // Evaluate: x * y
    let xy = Expr::Binary {
        op: BinOp::Mul,
        left: Box::new(Expr::Var("x".into())),
        right: Box::new(Expr::Var("y".into())),
    };
    println!("x * y = {:.6}", eval(&xy, &env).unwrap()); // 21.000000
}
```

## API

### AST Types
- **`Expr::Literal(f64)`** — Numeric constant
- **`Expr::Var(String)`** — Variable reference (looked up in `Env`)
- **`Expr::Binary { op, left, right }`** — Binary operation (`Add`, `Sub`, `Mul`, `Div`, `Pow`)
- **`Expr::UnaryNeg(Box<Expr>)`** — Unary negation
- **`Expr::Call { name, args }`** — Function call

### Environment
- **`Env::new()`** — Built-in vars (`pi`, `e`) and functions (`sqrt`, `abs`, `sin`, `cos`, `max`, `min`)
- **`Env::with(name, value)`** — Builder pattern for adding variables

### Evaluation
- **`eval(expr, env) → Result<f64, EvalError>`** — Evaluate an AST against an environment

## Architecture Notes

This crate provides the expression evaluation layer for SuperInstance's configuration and rule engine. Variables map to system parameters, and functions map to sensor data transforms. Together with `expr-bytecode` (for repeated evaluation), it forms the computational core of the γ + η = C decision pipeline.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full design.

## References

- Appel, A. W. (2004). *Modern Compiler Implementation* (2nd ed.). Cambridge University Press. Chapter 4 on abstract syntax.
- Kamin, S. (2010). *Programming Languages: An Interpreter-Based Approach*.
- Okasaki, C. (1999). *Purely Functional Data Structures*. Cambridge University Press.

## License

MIT
