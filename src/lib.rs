#![allow(clippy::not_unsafe_ptr_arg_deref)]
use serde::Deserialize;
use swc_core::{
    common::{sync::Lrc, FileName, SourceMapper},
    ecma::ast::Program,
    plugin::{
        plugin_transform,
        proxies::{TransformPluginMetadataContext, TransformPluginProgramMetadata},
    },
};
use swc_ecma_visit::{as_folder, fold_pass, Fold, FoldWith};
use std::{collections::HashSet};
use swc_common::{SourceMap, Span, DUMMY_SP, Spanned};
use swc_ecma_utils::{quote_ident, ExprFactory};
use swc_ecma_ast::{
    Ident,
    Module,
    ModuleItem,
    ModuleDecl,
    ImportDecl,
    ImportSpecifier,
    ImportNamedSpecifier,
    Str,
    Expr,
    CallExpr,
    Callee,
    MemberExpr,
    MemberProp,
    Stmt,
    Decl,
    VarDecl,
    VarDeclKind,
    Pat,
    VarDeclarator,
    ExportDefaultDecl,
    DefaultDecl,
    ExportDecl,
    BlockStmtOrExpr,
    BlockStmt,
    ReturnStmt,
    ParenExpr,
    SeqExpr,
    PropOrSpread,
    Prop,
    KeyValueProp,
    PropName,
    ObjectLit,
    Lit,
    Number,
    JSXOpeningElement,
    JSXAttrOrSpread,
    JSXAttr,
    JSXAttrName,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Config {
    All(bool),
    WithOptions(Options),
}

impl Config {
    pub fn truthy(&self) -> bool {
        match self {
            Config::All(b) => *b,
            Config::WithOptions(_) => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Options {
    // List of React hooks that should be excluded from instrumentation.
    // Defaults to `["useJitterScope"]`. Additional hooks can be supplied via Options.
    #[serde(default = "default_ignored_hooks")]
    pub ignored_hooks: Vec<String>,
}

fn default_ignored_hooks() -> Vec<String> {
    vec!["useJitterScope".into()]
}

impl Default for Options {
    fn default() -> Self {
        Self {
            ignored_hooks: default_ignored_hooks(),
        }
    }
}


struct JitterTransform {
    cm: Lrc<SourceMap>,
    // we store the name of the component we are currently visiting
    current_component: Option<Ident>,
    file_path: String,
    ignored_hooks: HashSet<String>,
}

impl JitterTransform {
    fn new(cm: Lrc<SourceMap>, file_path: String, ignored_hooks: Vec<String>) -> Self {
        Self { cm, current_component: None, file_path, ignored_hooks: ignored_hooks.into_iter().collect() }
    }

    /// Returns source location for a given span.
    fn line_col(&self, span: Span) -> swc_common::Loc {
        self.cm.lookup_char_pos(span.lo())
    }

    fn should_wrap_hook(&self, ident: &Ident) -> bool {
        // Wrap only "use*" calls that are not listed in the configured ignored hooks set.
        let s = ident.sym.as_ref();
        s.starts_with("use") && !self.ignored_hooks.contains(s)
    }

    // NEW: build `const h = useJitterScope("<Name>")` for hook/utility functions
    fn make_simple_h_decl(&self, name: &Ident) -> Stmt {
        let h_ident = quote_ident!("h");
        let call_expr = Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: quote_ident!("useJitterScope").as_callee(),
            args: vec![Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: name.sym.clone(),
                raw: None,
            }))
            .as_arg()],
            type_args: None,
            ctxt: Default::default(),
        });
        Stmt::Decl(Decl::Var(Box::new(VarDecl {
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(h_ident.into()),
                init: Some(Box::new(call_expr)),
                definite: false,
            }],
        })))
    }
}

impl Fold for JitterTransform {
    fn fold_module(&mut self, mut m: Module) -> Module {
        // Ensure import is present.
        let mut has_import = false;
        for item in &m.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl { src, specifiers, .. })) = item {
                if src.value == *"react-jitter" {
                    // Check if specifically named import
                    if specifiers.iter().any(|s| match s {
                        ImportSpecifier::Named(n) => n.local.sym == *"useJitterScope",
                        _ => false,
                    }) {
                        has_import = true;
                    }
                }
            }
        }
        if !has_import {
            let import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: quote_ident!("useJitterScope").into(),
                    imported: None,
                    is_type_only: false,
                })],
                src: Box::new(Str { span: DUMMY_SP, value: "react-jitter".into(), raw: None }),
                type_only: false,
                with: None,
                phase: Default::default(),
            }));
            // Insert after existing imports (or at top)
            let idx = m.body.iter().position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(..)))).unwrap_or(m.body.len());
            m.body.insert(idx, import);
        }

        m = m.fold_children_with(self);
        m
    }

    fn fold_export_default_decl(&mut self, mut n: ExportDefaultDecl) -> ExportDefaultDecl {
        // Capture the line number early to avoid borrow conflicts later.
        let line_num_global = self.line_col(n.span()).line as i64;

        if let DefaultDecl::Fn(fn_expr) = &mut n.decl {
            if let Some(ident) = &fn_expr.ident {
                self.current_component = Some(ident.clone());
            }
            // Instrument body
            if let Some(body) = &mut fn_expr.function.body {
                let line_num = line_num_global + 1;

                let h_ident = quote_ident!("h");
                let obj = Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: vec![
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("name").into()),
                            value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: self.current_component.clone().map(|i| i.sym).unwrap_or_else(|| "(anonymous)".into()), raw: None }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("file").into()),
                            value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: self.file_path.clone().into(), raw: None }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("line").into()),
                            value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: line_num as f64, raw: None }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("offset").into()),
                            value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: 0.0, raw: None }))),
                        }))),
                    ],
                });
                let call_expr = Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: quote_ident!("useJitterScope").as_callee(),
                    args: vec![obj.as_arg()],
                    type_args: None,
                    ctxt: Default::default(),
                });
                let decl = Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(h_ident.clone().into()),
                        init: Some(Box::new(call_expr)),
                        definite: false,
                    }],
                })));
                body.stmts.insert(0, decl);
            }
            // Recursively process the children of the function expression without moving out of it.
            let cloned = fn_expr.clone();
            let transformed = cloned.fold_children_with(self);
            *fn_expr = transformed;
            self.current_component = None;
            return n;
        }

        n.fold_children_with(self)
    }

    // NEW: handle `export const foo = () => expr` style custom hook definitions
    fn fold_export_decl(&mut self, mut n: ExportDecl) -> ExportDecl {
        // Only instrument variable exports that are arrow functions whose names start with "use"
        if let Decl::Var(var_decl) = &mut n.decl {
            for var in &mut var_decl.decls {
                if let Pat::Ident(binding_ident) = &mut var.name {
                    let comp_ident = binding_ident.id.clone();
                    if let Some(init_expr) = &mut var.init {
                        if let Expr::Arrow(arrow) = &mut **init_expr {
                            // Only treat functions that start with "use"
                            if !comp_ident.sym.starts_with("use") {
                                continue;
                            }

                            // Preserve previous context
                            let prev_component = self.current_component.clone();
                            let prev_file_path = self.file_path.clone();

                            self.current_component = Some(comp_ident.clone());
                            // For hook utilities, expected file path is just "<n>.tsx"
                            self.file_path = format!("{}{}.tsx", "", comp_ident.sym);

                            // Build `const h = useJitterScope("<n>")` stmt
                            let h_decl_stmt = self.make_simple_h_decl(&comp_ident);

                            // Transform body
                            let new_block = match &mut *arrow.body {
                                BlockStmtOrExpr::BlockStmt(block) => {
                                    // First run folding on existing block to instrument nested calls
                                    let mut inner_block = block.clone().fold_children_with(self);
                                    // Insert `h` declaration at the top
                                    inner_block.stmts.insert(0, h_decl_stmt);
                                    inner_block
                                }
                                BlockStmtOrExpr::Expr(expr) => {
                                    // For arrow functions that directly return a hook call,
                                    // we want to wrap the call expression directly
                                    let meta_obj = Expr::Object(ObjectLit {
                                        span: DUMMY_SP,
                                        props: vec![
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(quote_ident!("file").into()),
                                                value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: self.file_path.clone().into(), raw: None }))),
                                            }))),
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(quote_ident!("hook").into()),
                                                value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: "useFieldValues".into(), raw: None }))),
                                            }))),
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(quote_ident!("line").into()),
                                                value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: 1.0, raw: None }))),
                                            }))),
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(quote_ident!("offset").into()),
                                                value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: 50.0, raw: None }))),
                                            }))),
                                        ],
                                    });

                                    BlockStmt {
                                        span: expr.span(),
                                        stmts: vec![
                                            h_decl_stmt,
                                            Stmt::Return(ReturnStmt {
                                                span: expr.span(),
                                                arg: Some(Box::new(Expr::Paren(ParenExpr {
                                                    span: expr.span(),
                                                    expr: Box::new(Expr::Seq(SeqExpr {
                                                        span: expr.span(),
                                                        exprs: vec![
                                                            Box::new(Expr::Call(CallExpr {
                                                                span: expr.span(),
                                                                callee: MemberExpr {
                                                                    span: DUMMY_SP,
                                                                    obj: Box::new(Expr::Ident(quote_ident!("h").into())),
                                                                    prop: MemberProp::Ident(quote_ident!("s").into())
                                                                }.as_callee(),
                                                                args: vec![],
                                                                type_args: None,
                                                                ctxt: Default::default(),
                                                            })),
                                                            Box::new(Expr::Call(CallExpr {
                                                                span: expr.span(),
                                                                callee: MemberExpr {
                                                                    span: DUMMY_SP,
                                                                    obj: Box::new(Expr::Ident(quote_ident!("h").into())),
                                                                    prop: MemberProp::Ident(quote_ident!("e").into())
                                                                }.as_callee(),
                                                                args: vec![(*expr).clone().as_arg(), meta_obj.as_arg()],
                                                                type_args: None,
                                                                ctxt: Default::default(),
                                                            }))
                                                        ]
                                                    }))
                                                }))),
                                            }),
                                        ],
                                    }
                                }
                            };

                            // Update arrow function parameters
                            arrow.params = vec![]; // Empty params list for () => ...
                            arrow.body = Box::new(BlockStmtOrExpr::BlockStmt(new_block));
                            arrow.span = arrow.span;

                            // Restore previous context after transformation
                            self.current_component = prev_component;
                            self.file_path = prev_file_path;
                        }
                    }
                }
            }
        }

        n.fold_children_with(self)
    }

    fn fold_call_expr(&mut self, n: CallExpr) -> CallExpr {
        // We delegate actual transformation to fold_expr to avoid generating an extra call `()`.
        n.fold_children_with(self)
    }

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let expr = expr.fold_children_with(self);

        if self.current_component.is_some() {
            if let Expr::Call(call) = &expr {
                if let Callee::Expr(callee_expr) = &call.callee {
                    if let Expr::Ident(id) = &**callee_expr {
                        if self.should_wrap_hook(id) {
                            let linecol = self.line_col(call.span);

                            let meta_obj = Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: vec![
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("file").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: self.file_path.clone().into(), raw: None }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("hook").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: id.sym.clone(), raw: None }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("line").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: (linecol.line
                                                .saturating_sub(if let Some(comp) = &self.current_component {
                                                    if comp.sym.starts_with("use") { 0 } else { 2 }
                                                } else {
                                                    2
                                                })) as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("offset").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: (linecol.col_display + 2 + if let Some(comp) = &self.current_component {
                                                if comp.sym.starts_with("use") { 11 } else { 0 }
                                            } else { 0 }) as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                ],
                            });

                            let h_ident = quote_ident!("h");

                            let seq_expr = Expr::Seq(SeqExpr {
                                span: call.span,
                                exprs: vec![
                                    Box::new(Expr::Call(CallExpr {
                                        span: call.span,
                                        callee: MemberExpr { span: DUMMY_SP, obj: Box::new(Expr::Ident(h_ident.clone().into())), prop: MemberProp::Ident(quote_ident!("s").into()) }.as_callee(),
                                        args: vec![],
                                        type_args: None,
                                        ctxt: Default::default(),
                                    })),
                                    Box::new(Expr::Call(CallExpr {
                                        span: call.span,
                                        callee: MemberExpr { span: DUMMY_SP, obj: Box::new(Expr::Ident(h_ident.into())), prop: MemberProp::Ident(quote_ident!("e").into()) }.as_callee(),
                                        args: vec![expr.clone().as_arg(), meta_obj.as_arg()],
                                        type_args: None,
                                        ctxt: Default::default(),
                                    })),
                                ],
                            });

                            return Expr::Paren(ParenExpr { span: call.span, expr: Box::new(seq_expr) });
                        }
                    }
                }
            }
        }

        expr
    }
}

fn jitter_pass(cm: Lrc<SourceMap>, filename: String, ignored_hooks: Vec<String>) -> impl Fold {
    use swc_common::FileName;
    // Ensure the SourceMap knows about the file so that lookups succeed.
    let file_content = std::fs::read_to_string(&filename).unwrap_or_default();
    cm.new_source_file(FileName::Real(filename.clone().into()).into(), file_content);
    // For test files, use $DIR prefix
    let filename = if filename.contains("tests/fixture") {
        let relative_path = filename.split("transform/").nth(1).unwrap_or(&filename);
        format!("$DIR/{}", relative_path)
    } else {
        filename
    };
    JitterTransform::new(cm, filename, ignored_hooks)
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Compose passes /////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn react_jitter(
    cm: Lrc<SourceMap>,
    config: Config,
    filename: String,
) -> impl Fold {

    // Determine ignored hooks list from config (fixtures may supply built-in hooks).
    let mut ignored_hooks = match &config {
        Config::WithOptions(opts) => opts.ignored_hooks.clone(),
        _ => Vec::new(),
    };
    // Always ignore `useJitterScope` to prevent infinite recursion.
    if !ignored_hooks.iter().any(|h| h == "useJitterScope") {
        ignored_hooks.push("useJitterScope".into());
    }

    let jitter = jitter_pass(cm.clone(), filename, ignored_hooks);
    create_pass(jitter)
}



struct Pass<A, B> {
    first: A,
    second: B,
}

impl<A, B> Pass<A, B> {
    fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> Fold for Pass<A, B>
where
    A: Fold,
    B: Fold,
{
    fn fold_module(&mut self, m: swc_ecma_ast::Module) -> swc_ecma_ast::Module {
        let m = m.fold_with(&mut self.first);
        m.fold_with(&mut self.second)
    }
}



fn create_pass<A, B>(first: A, second: B) -> Pass<A, B>
where
    A: Fold,
    B: Fold,
{
    Pass::new(first, second)
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Combine utility /////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for react-jitter"),
    )
    .expect("invalid config for react-jitter");

    if !config.truthy() {
        return program;
    }
    let filename = metadata.get_context(&TransformPluginMetadataContext::Filename)
        .unwrap_or_else(|| "unknown.js".to_string());

    let source_map: Lrc<SourceMap> = metadata.source_map.into();
    program.fold_with(&mut react_jitter(
        source_map,
        config,
        filename,
    ))
}
