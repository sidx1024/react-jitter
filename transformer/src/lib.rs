use serde::{Deserialize, Serialize};
use swc_core::common::util::take::Take;
use swc_core::common::{SourceMap, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub filename: String,
}

struct ReactJitter<'a> {
    file: &'a str,
    cm: Option<&'a SourceMap>,
    imported: bool,
}

impl<'a> VisitMut for ReactJitter<'a> {
    fn visit_mut_module(&mut self, m: &mut Module) {
        self.imported = m.body.iter().any(|i| {
            matches!(i,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl { src, .. }))
                    if src.value == *"react-jitter")
        });
        if !self.imported {
            let import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: Ident::new("useJitterScope".into(), DUMMY_SP),
                    imported: None,
                    is_type_only: false,
                })],
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: "react-jitter".into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
            }));
            m.body.insert(0, import);
            self.imported = true;
        }
        m.visit_mut_children_with(self);
    }

    fn visit_mut_fn_decl(&mut self, n: &mut FnDecl) {
        let name = n.ident.sym.to_string();
        if is_component(&name) {
            if let Some(body) = &mut n.function.body {
                body.stmts.insert(0, scope_stmt(&name));
            }
        }
        n.visit_mut_children_with(self);
    }

    fn visit_mut_var_declarator(&mut self, n: &mut VarDeclarator) {
        if let Some(init) = &mut n.init {
            if let Expr::Arrow(a) = &mut **init {
                if let Some(name) = var_name(&n.name) {
                    if is_component(&name) {
                        if let BlockStmtOrExpr::Expr(e) = &mut *a.body {
                            *a.body = BlockStmtOrExpr::BlockStmt(BlockStmt {
                                span: DUMMY_SP,
                                stmts: vec![Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(e.take()),
                                })],
                            });
                        }
                        if let BlockStmtOrExpr::BlockStmt(b) = &mut *a.body {
                            b.stmts.insert(0, scope_stmt(&name));
                        }
                    }
                }
            } else if let Expr::Call(call) = &mut **init {
                if let Callee::Expr(callee) = &call.callee {
                    if let Expr::Ident(id) = &**callee {
                        let hook = id.sym.to_string();
                        if is_custom_hook(&hook) {
                            let (line, col) = if let Some(cm) = self.cm {
                                let loc = cm.lookup_char_pos(call.span.lo());
                                (loc.line as i64, (loc.col_display + 1) as i64)
                            } else {
                                (0, 0)
                            };
                            let id = hash_id(self.file, line, col);
                            let meta = meta_obj(&id, self.file, &hook, line, col);
                            let orig = call.take();
                            let id_expr = str_lit(&id);
                            let seq = Expr::Seq(SeqExpr {
                                span: DUMMY_SP,
                                exprs: vec![
                                    Box::new(start_call()),
                                    Box::new(end_call(orig, meta, id_expr)),
                                ],
                            });
                            **init = seq;
                            return;
                        }
                    }
                }
            }
        }
        n.visit_mut_children_with(self);
    }
}

fn is_component(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

fn var_name(p: &Pat) -> Option<String> {
    if let Pat::Ident(b) = p {
        Some(b.id.sym.to_string())
    } else {
        None
    }
}

const BUILT_INS: &[&str] = &[
    "useState",
    "useEffect",
    "useLayoutEffect",
    "useMemo",
    "useCallback",
    "useRef",
    "useReducer",
    "useContext",
    "useJitterScope",
];

fn is_custom_hook(name: &str) -> bool {
    name.starts_with("use")
        && name
            .get(3..)
            .map(|s| {
                s.chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
        && !BUILT_INS.contains(&name)
}

fn scope_stmt(name: &str) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: Ident::new("h".into(), DUMMY_SP),
                type_ann: None,
            }),
            init: Some(Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                    "useJitterScope".into(),
                    DUMMY_SP,
                )))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: name.into(),
                        raw: None,
                    }))),
                }],
                type_args: None,
            }))),
            definite: false,
        }],
    })))
}

fn start_call() -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new("h".into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("s".into(), DUMMY_SP)),
        }))),
        args: vec![],
        type_args: None,
    })
}

fn end_call(inner: CallExpr, meta: Expr, id: Expr) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new("h".into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("e".into(), DUMMY_SP)),
        }))),
        args: vec![
            ExprOrSpread {
                spread: None,
                expr: Box::new(id),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Call(inner)),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(meta),
            },
        ],
        type_args: None,
    })
}

fn meta_obj(id: &str, file: &str, hook: &str, line: i64, offset: i64) -> Expr {
    Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props: vec![
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("id".into(), DUMMY_SP)),
                value: Box::new(str_lit(id)),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("file".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: file.into(),
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("hook".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: hook.into(),
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("line".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: line as f64,
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(Ident::new("offset".into(), DUMMY_SP)),
                value: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: offset as f64,
                    raw: None,
                }))),
            }))),
        ],
    })
}

fn str_lit(value: &str) -> Expr {
    Expr::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: value.into(),
        raw: None,
    }))
}

fn hash_id(file: &str, line: i64, offset: i64) -> String {
    let input = format!("{}:{}:{}", file, line, offset);
    let hash = xxh3_64(input.as_bytes());
    format!("{:08x}", hash)
}

#[plugin_transform]
pub fn transform_program(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = metadata
        .get_transform_plugin_config()
        .and_then(|s| serde_json::from_str::<Config>(&s).ok())
        .unwrap_or_default();
    transform(program, &config.filename, None)
}

pub fn transform(mut program: Program, filename: &str, cm: Option<&SourceMap>) -> Program {
    program.visit_mut_with(&mut ReactJitter {
        file: filename,
        cm,
        imported: false,
    });
    program
}
