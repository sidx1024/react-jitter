use swc_core::common::DUMMY_SP;
use swc_ecma_ast::{
    BinExpr, BinaryOp, Bool, Expr, Ident, Lit, Str, UnaryExpr, UnaryOp,
};

/// Keys commonly present on Jest/Vitest mock functions (or compatible shims).
///
/// This list is intended to detect `vi.fn()` / `jest.fn()` style mocks by
/// checking if ANY of these keys exist on the hook function at runtime.
const MOCK_KEYS: [&str; 2] = [
    "mockImplementation",
    "mockReturnValue",
];

fn build_mock_key_or_chain(hook_ident: &Ident) -> Expr {
    let mut acc: Option<Expr> = None;

    for key in MOCK_KEYS.iter() {
        // Builds `"mockClear" in hookFn`
        let in_expr = Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::In,
            left: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: (*key).into(),
                raw: None,
            }))),
            right: Box::new(Expr::Ident(hook_ident.clone())),
        });

        // Builds `(<prev>) || ("key" in hookFn)`
        acc = Some(match acc {
            None => in_expr,
            Some(prev) => Expr::Bin(BinExpr {
                span: DUMMY_SP,
                op: BinaryOp::LogicalOr,
                left: Box::new(prev),
                right: Box::new(in_expr),
            }),
        });
    }

    acc.unwrap_or_else(|| Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: false })))
}

/// Builds an SWC AST expression equivalent to:
///
/// ```js
/// (typeof hookFn === "function") && (
///   ("mockClear" in hookFn) || ("mockReset" in hookFn) || ...
/// )
/// ```
///
/// The `typeof` guard is important because `"x" in undefined` throws at runtime.
pub fn build_is_mocked_expr(hook_ident: &Ident) -> Expr {
    let typeof_expr = Expr::Unary(UnaryExpr {
        span: DUMMY_SP,
        op: UnaryOp::TypeOf,
        arg: Box::new(Expr::Ident(hook_ident.clone())),
    });

    let is_function_expr = Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::EqEqEq,
        left: Box::new(typeof_expr),
        right: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "function".into(),
            raw: None,
        }))),
    });

    let keys_or_chain = build_mock_key_or_chain(hook_ident);

    Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::LogicalAnd,
        left: Box::new(is_function_expr),
        right: Box::new(keys_or_chain),
    })
}

