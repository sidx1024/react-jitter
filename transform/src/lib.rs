use serde::Deserialize;
use serde_json;
use std::collections::HashSet;
use swc_core::common::errors::SourceMapper;
use swc_core::common::{Loc, Span, Spanned, SyntaxContext, DUMMY_SP};
use swc_core::ecma::ast::Program;
use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform, proxies::PluginSourceMapProxy,
    proxies::TransformPluginProgramMetadata,
};
use swc_ecma_ast::*;
use swc_ecma_utils::{quote_ident, ExprFactory};
use swc_ecma_visit::{VisitMut, VisitMutWith};

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
    cm: PluginSourceMapProxy,
    current_component: Option<Ident>,
    file_path: String,
    ignored_hooks: HashSet<String>,
}

impl JitterTransform {
    fn new(cm: PluginSourceMapProxy, file_path: String, ignored_hooks: Vec<String>) -> Self {
        Self {
            cm,
            current_component: None,
            file_path,
            ignored_hooks: ignored_hooks.into_iter().collect(),
        }
    }

    fn generate_location_hash(&self, file: &str, line: f64, offset: f64) -> String {
        use sha2::{Digest, Sha256};
        let input = format!("{}:{}:{}", file, line, offset);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..4]) // Use first 8 characters (4 bytes) of hash
    }

    fn line_col(&self, span: Span) -> Loc {
        self.cm.lookup_char_pos(span.lo())
    }

    fn should_wrap_hook(&self, ident: &Ident) -> bool {
        let s = ident.sym.as_ref();
        s.starts_with("use") && !self.ignored_hooks.contains(s)
    }
}

impl VisitMut for JitterTransform {
    fn visit_mut_module(&mut self, m: &mut Module) {
        let mut has_import = false;
        for item in m.body.iter() {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                src, specifiers, ..
            })) = item
            {
                if src.value == *"react-jitter" {
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
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: "react-jitter".into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
                phase: Default::default(),
            }));
            let idx = m
                .body
                .iter()
                .position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(..))))
                .unwrap_or(m.body.len());
            m.body.insert(idx, import);
        }
        m.visit_mut_children_with(self);
    }

    fn visit_mut_export_default_decl(&mut self, n: &mut ExportDefaultDecl) {
        let line_num_global = self.line_col(n.span).line as i64;
        if let DefaultDecl::Fn(fn_expr) = &mut n.decl {
            if let Some(ident) = &fn_expr.ident {
                self.current_component = Some(ident.clone());
            } else {
                // Use a placeholder to enable instrumentation for anonymous components
                self.current_component = Some(quote_ident!("(anonymous)").into());
            }
            if let Some(body) = &mut fn_expr.function.body {
                let linecol = self.line_col(n.span);
                let hash = self.generate_location_hash(
                    &self.file_path,
                    linecol.line as f64,
                    linecol.col_display as f64,
                );

                let meta_obj = Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: vec![
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("name").into()),
                            value: Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: self
                                    .current_component
                                    .as_ref()
                                    .map(|i| i.sym.clone())
                                    .unwrap_or_else(|| "(anonymous)".into()),
                                raw: None,
                            }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("id").into()),
                            value: Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: hash.into(),
                                raw: None,
                            }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("file").into()),
                            value: Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: self.file_path.clone().into(),
                                raw: None,
                            }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("line").into()),
                            value: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: linecol.line as f64,
                                raw: None,
                            }))),
                        }))),
                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("offset").into()),
                            value: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: linecol.col_display as f64,
                                raw: None,
                            }))),
                        }))),
                    ],
                });

                let h_decl = Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(quote_ident!("h").into()),
                        init: Some(Box::new(Expr::Call(CallExpr {
                            span: DUMMY_SP,
                            callee: quote_ident!("useJitterScope").as_callee(),
                            args: vec![meta_obj.as_arg()],
                            type_args: None,
                            ctxt: SyntaxContext::empty(),
                        }))),
                        definite: false,
                    }],
                    ctxt: SyntaxContext::empty(),
                })));
                body.stmts.insert(0, h_decl);
            }
            fn_expr.visit_mut_children_with(self);
            self.current_component = None;
            return;
        }
        n.visit_mut_children_with(self);
    }

    fn visit_mut_export_decl(&mut self, n: &mut ExportDecl) {
        if let Decl::Var(var_decl) = &mut n.decl {
            for var in &mut var_decl.decls {
                if let Pat::Ident(binding_ident) = &var.name {
                    let comp_ident = binding_ident.id.clone();
                    if let Some(init_expr) = &mut var.init {
                        if let Expr::Arrow(arrow) = &mut **init_expr {
                            if !comp_ident.sym.starts_with("use") {
                                continue;
                            }
                            let prev_component = self.current_component.clone();
                            let prev_file_path = self.file_path.clone();
                            self.current_component = Some(comp_ident.clone());
                            self.file_path = format!("{}{}.tsx", "", comp_ident.sym);

                            let linecol = self.line_col(arrow.span);
                            let hash = self.generate_location_hash(
                                &self.file_path,
                                linecol.line as f64,
                                linecol.col_display as f64,
                            );
                            let meta_obj = Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: vec![
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("name").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: comp_ident.sym.clone(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("id").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: hash.into(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("file").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: self.file_path.clone().into(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("line").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.line as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("offset").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.col_display as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                ],
                            });

                            let h_decl_stmt = Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                span: DUMMY_SP,
                                kind: VarDeclKind::Const,
                                declare: false,
                                decls: vec![VarDeclarator {
                                    span: DUMMY_SP,
                                    name: Pat::Ident(quote_ident!("h").into()),
                                    init: Some(Box::new(Expr::Call(CallExpr {
                                        span: DUMMY_SP,
                                        callee: quote_ident!("useJitterScope").as_callee(),
                                        args: vec![meta_obj.as_arg()],
                                        type_args: None,
                                        ctxt: SyntaxContext::empty(),
                                    }))),
                                    definite: false,
                                }],
                                ctxt: SyntaxContext::empty(),
                            })));

                            arrow.body = Box::new(match &mut *arrow.body {
                                BlockStmtOrExpr::BlockStmt(block) => {
                                    block.visit_mut_with(self);
                                    let mut inner_block = block.clone();
                                    inner_block.stmts.insert(0, h_decl_stmt);
                                    BlockStmtOrExpr::BlockStmt(inner_block)
                                }
                                BlockStmtOrExpr::Expr(expr) => {
                                    expr.visit_mut_with(self);

                                    BlockStmtOrExpr::BlockStmt(BlockStmt {
                                        span: expr.span(),
                                        stmts: vec![
                                            h_decl_stmt,
                                            Stmt::Return(ReturnStmt {
                                                span: expr.span(),
                                                arg: Some(expr.clone()),
                                            }),
                                        ],
                                        ctxt: SyntaxContext::empty(),
                                    })
                                }
                            });
                            arrow.params = vec![]; // Empty params for () => ...
                            self.current_component = prev_component;
                            self.file_path = prev_file_path;
                        }
                    }
                }
            }
        }
        n.visit_mut_children_with(self);
    }

    fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
        n.visit_mut_children_with(self);
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);
        if self.current_component.is_some() {
            if let Expr::Call(call) = &*expr {
                if let Callee::Expr(callee_expr) = &call.callee {
                    if let Expr::Ident(id) = &**callee_expr {
                        if self.should_wrap_hook(id) {
                            let linecol = self.line_col(call.span);
                            let hook_meta = Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: vec![
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("id").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: self
                                                .generate_location_hash(
                                                    &self.file_path,
                                                    linecol.line as f64,
                                                    linecol.col_display as f64,
                                                )
                                                .into(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("hook").into()),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: id.sym.clone(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("line").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.line as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("offset").into()),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.col_display as f64,
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
                                        callee: MemberExpr {
                                            span: DUMMY_SP,
                                            obj: Box::new(Expr::Ident(h_ident.clone().into())),
                                            prop: MemberProp::Ident(quote_ident!("s").into()),
                                        }
                                        .as_callee(),
                                        args: vec![Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: self
                                                .generate_location_hash(
                                                    &self.file_path,
                                                    linecol.line as f64,
                                                    linecol.col_display as f64,
                                                )
                                                .into(),
                                            raw: None,
                                        }))
                                        .as_arg()],
                                        type_args: None,
                                        ctxt: SyntaxContext::empty(),
                                    })),
                                    Box::new(Expr::Call(CallExpr {
                                        span: call.span,
                                        callee: MemberExpr {
                                            span: DUMMY_SP,
                                            obj: Box::new(Expr::Ident(h_ident.into())),
                                            prop: MemberProp::Ident(quote_ident!("e").into()),
                                        }
                                        .as_callee(),
                                        args: vec![expr.clone().as_arg(), hook_meta.as_arg()],
                                        type_args: None,
                                        ctxt: SyntaxContext::empty(),
                                    })),
                                ],
                            });
                            *expr = Expr::Paren(ParenExpr {
                                span: call.span,
                                expr: Box::new(seq_expr),
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn jitter_pass(
    cm: PluginSourceMapProxy,
    filename: String,
    ignored_hooks: Vec<String>,
) -> impl VisitMut {
    JitterTransform::new(cm, filename, ignored_hooks)
}
// Plugin entrypoint for SWC
#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    // Parse plugin config
    let config: Config = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for react-jitter"),
    )
    .expect("invalid config for react-jitter");

    // Skip if plugin disabled
    if !config.truthy() {
        return program;
    }

    // Determine filename and normalize path
    let filename = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap_or_else(|| "unknown.js".to_string());

    let cwd = metadata
        .get_context(&TransformPluginMetadataContextKind::Cwd)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        });

    let filename = if let Some(cwd) = cwd {
        let abs = {
            let p = std::path::Path::new(&filename);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::path::Path::new(&cwd).join(p)
            }
        };
        if let Ok(stripped) = abs.strip_prefix(&cwd) {
            let stripped = stripped.to_string_lossy();
            let stripped = stripped
                .trim_start_matches('/')
                .strip_prefix("transform/")
                .unwrap_or(&stripped);
            format!("$DIR/{}", stripped)
        } else {
            filename.clone()
        }
    } else {
        filename.clone()
    };

    // Clone SourceMap proxy
    let cm: PluginSourceMapProxy = metadata.source_map.clone();

    // Apply jitter_pass visitor
    program.visit_mut_with(&mut jitter_pass(
        cm,
        filename,
        match &config {
            Config::WithOptions(opts) => opts.ignored_hooks.clone(),
            _ => Vec::new(),
        },
    ));

    program
}
