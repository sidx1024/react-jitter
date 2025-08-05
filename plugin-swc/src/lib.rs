use serde::Deserialize;
use std::collections::HashSet;
use glob::Pattern;
use swc_core::common::errors::SourceMapper;
use swc_core::common::{Loc, Span, Spanned, SyntaxContext, DUMMY_SP};
use swc_core::ecma::ast::Program;
use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform, proxies::PluginSourceMapProxy,
    proxies::TransformPluginProgramMetadata,
};
use swc_ecma_ast::*;
use swc_ecma_utils::{quote_ident, ExprFactory};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};

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

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Options {
    #[serde(default = "default_ignored_hooks")]
    pub ignoreHooks: Vec<String>,
    #[serde(default = "default_exclude_patterns")]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub includeArguments: bool,
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        // Default patterns
        "**/node_modules/**".into(),  // Matches any node_modules directory at any depth
    ]
}

fn default_ignored_hooks() -> Vec<String> {
    vec![
        // Project-specific hooks
        "useJitterScope".into(),
        
        // Basic React Hooks
        "useState".into(),
        "useEffect".into(),
        // "useContext".into(),
        // "useReducer".into(),
        "useCallback".into(),
        "useMemo".into(),
        "useRef".into(),
        "useImperativeHandle".into(),
        "useLayoutEffect".into(),
        "useDebugValue".into(),
        "useId".into(),

        // React Suspense Hooks
        "useDeferredValue".into(),
        "useTransition".into(),

        // React Cache/Resource Hooks
        "useCacheRefresh".into(),
        "useInsertionEffect".into(),
        "useSyncExternalStore".into(),
    ]
}

struct ReactFnAnalyzer {
    should_instrument: bool,
}

impl ReactFnAnalyzer {
    fn new() -> Self {
        Self {
            should_instrument: false,
        }
    }

    fn analyze_fn<F>(&mut self, mut op: F) -> bool
    where
        F: FnMut(&mut Self),
    {
        op(self);
        self.should_instrument
    }
}

impl Visit for ReactFnAnalyzer {
    fn visit_jsx_element(&mut self, _: &JSXElement) {
        self.should_instrument = true;
    }

    fn visit_jsx_fragment(&mut self, _: &JSXFragment) {
        self.should_instrument = true;
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        if self.should_instrument {
            return;
        }
        if let Callee::Expr(expr) = &n.callee {
            if let Expr::Ident(id) = &**expr {
                let s = id.sym.as_ref();
                if s.starts_with("use")
                    && s.len() > 3 {
                        if let Some(c) = s.chars().nth(3) {
                            if c.is_uppercase() {
                                self.should_instrument = true;
                                return;
                            }
                        }
                    }
            }
        }
        n.visit_children_with(self);
    }

    // Stop traversal at nested functions/classes
    fn visit_fn_decl(&mut self, _: &FnDecl) {}
    fn visit_fn_expr(&mut self, _: &FnExpr) {}
    fn visit_arrow_expr(&mut self, _: &ArrowExpr) {}
    fn visit_class_decl(&mut self, _: &ClassDecl) {}
    fn visit_class_expr(&mut self, _: &ClassExpr) {}
}


struct JitterTransform {
    cm: PluginSourceMapProxy,
    current_component: Option<Ident>,
    file_path: String,
    ignoreHooks: HashSet<String>,
    exclude_patterns: Vec<Pattern>,
    instrumented_any_function: bool,
    include_arguments: bool,
}

impl JitterTransform {
    fn new(cm: PluginSourceMapProxy, file_path: String, options: Options) -> Self {
        let compiled_patterns = options
            .exclude
            .into_iter()
            .filter_map(|p| Pattern::new(&p).ok())
            .collect();
        
        Self {
            cm,
            current_component: None,
            file_path,
            ignoreHooks: options.ignoreHooks.into_iter().collect(),
            exclude_patterns: compiled_patterns,
            instrumented_any_function: false,
            include_arguments: options.includeArguments,
        }
    }

    fn normalize_path(&self) -> String {
        // Convert Windows-style paths to Unix-style for consistent matching
        self.file_path.replace('\\', "/")
    }

    fn should_exclude_file(&self) -> bool {
        let normalized_path = self.normalize_path();
        for pattern in &self.exclude_patterns {
            if pattern.matches(&normalized_path) {
                return true;
            }
        }
        false
    }

    fn generate_location_hash(&self, file: &str, line: f64, offset: f64) -> String {
        use sha2::{Digest, Sha256};
        let input = format!("{file}:{line}:{offset}");
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
        if !s.starts_with("use") || self.ignoreHooks.contains(s) {
            return false;
        }

        s.len() > 3 && s.chars().nth(3).is_some_and(|c| c.is_uppercase())
    }

    fn instrument_function_body(&mut self, body: &mut BlockStmt, component_ident: &Ident, span: Span) {
        self.instrumented_any_function = true;
        let linecol = self.line_col(span);
        let hash = self.generate_location_hash(
            &self.file_path,
            linecol.line as f64,
            linecol.col_display as f64,
        );

        let props = vec![
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("name")),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: component_ident.sym.clone(),
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("id")),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: hash.into(),
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("file")),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: self.file_path.clone().into(),
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("line")),
                value: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: linecol.line as f64,
                    raw: None,
                }))),
            }))),
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(quote_ident!("offset")),
                value: Box::new(Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: linecol.col_display as f64,
                    raw: None,
                }))),
            }))),
        ];

        let meta_obj = Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
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

    fn wrap_in_h_re(&self, expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Call(CallExpr {
            span: expr.span(),
            callee: MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(quote_ident!("h").into())),
                prop: MemberProp::Ident(quote_ident!("re")),
            }
            .as_callee(),
            args: vec![expr.as_arg()],
            type_args: None,
            ctxt: SyntaxContext::empty(),
        }))
    }
}

impl VisitMut for JitterTransform {
    fn visit_mut_module(&mut self, m: &mut Module) {
        if self.should_exclude_file() {
            return;
        }

        m.visit_mut_children_with(self);

        if self.instrumented_any_function {
            let mut has_import = false;
            for item in m.body.iter() {
                if let ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    src, specifiers, ..
                })) = item
                {
                    if src.value == *"react-jitter/runtime"
                        && specifiers.iter().any(|s| match s {
                            ImportSpecifier::Named(n) => n.local.sym == *"useJitterScope",
                            _ => false,
                        }) {
                            has_import = true;
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
                        value: "react-jitter/runtime".into(),
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
        }
    }

    fn visit_mut_fn_decl(&mut self, n: &mut FnDecl) {
        let ident_name = n.ident.sym.as_ref();
        let is_component = ident_name.chars().next().is_some_and(|c| c.is_uppercase());

        if is_component
            && ReactFnAnalyzer::new().analyze_fn(|analyzer| {
                n.function.visit_with(analyzer);
            }) {
                let prev_component = self.current_component.clone();
                self.current_component = Some(n.ident.clone());

                if let Some(body) = &mut n.function.body {
                    self.instrument_function_body(body, &n.ident, n.function.span);
                }
                
                n.function.visit_mut_children_with(self);
                self.current_component = prev_component;
                return;
            }
        
        n.visit_mut_children_with(self);
    }

    fn visit_mut_export_default_decl(&mut self, n: &mut ExportDefaultDecl) {
        if let DefaultDecl::Fn(fn_expr) = &mut n.decl {
            let is_component = match &fn_expr.ident {
                Some(id) => id.sym.chars().next().is_some_and(|c| c.is_uppercase()),
                None => true, // Anonymous functions are often components
            };

            if is_component
                && ReactFnAnalyzer::new().analyze_fn(|analyzer| {
                    fn_expr.function.visit_with(analyzer);
                }) {
                    let ident = fn_expr.ident.clone().unwrap_or_else(|| quote_ident!("(anonymous)").into());
                    let prev_component = self.current_component.clone();
                    self.current_component = Some(ident.clone());
                    
                    if let Some(body) = &mut fn_expr.function.body {
                        self.instrument_function_body(body, &ident, fn_expr.function.span);
                    }
                    
                    fn_expr.visit_mut_children_with(self);
                    self.current_component = prev_component;
                    return;
                }
        }
        
        n.visit_mut_children_with(self);
    }

    fn visit_mut_export_decl(&mut self, n: &mut ExportDecl) {
        match &mut n.decl {
            Decl::Fn(fn_decl) => {
                let ident_name = fn_decl.ident.sym.as_ref();
                let is_component = ident_name.chars().next().is_some_and(|c| c.is_uppercase());

                if is_component
                    && ReactFnAnalyzer::new().analyze_fn(|analyzer| {
                        fn_decl.function.visit_with(analyzer);
                    }) {
                        let prev_component = self.current_component.clone();
                        self.current_component = Some(fn_decl.ident.clone());
            
                        if let Some(body) = &mut fn_decl.function.body {
                            self.instrument_function_body(body, &fn_decl.ident, fn_decl.function.span);
                        }
                        
                        fn_decl.function.visit_mut_children_with(self);
                        self.current_component = prev_component;
                        return;
                    }
                
                n.visit_mut_children_with(self);
            },
            Decl::Var(var_decl) => {
                for var in &mut var_decl.decls {
                    if let Pat::Ident(binding_ident) = &var.name {
                        let comp_ident = binding_ident.id.clone();
                        let ident_name = comp_ident.sym.as_ref();
                        let is_component = ident_name.chars().next().is_some_and(|c| c.is_uppercase());

                        if is_component {
                            if let Some(init_expr) = &mut var.init {
                                if ReactFnAnalyzer::new().analyze_fn(|analyzer| {
                                    init_expr.visit_with(analyzer);
                                }) {
                                    self.instrumented_any_function = true;
                                    let prev_component = self.current_component.clone();
                                    self.current_component = Some(comp_ident.clone());
                                   
                                    match &mut **init_expr {
                                        Expr::Arrow(arrow) => {
                                             let h_decl_stmt = {
                                                let linecol = self.line_col(arrow.span);
                                                let hash = self.generate_location_hash(
                                                    &self.file_path,
                                                    linecol.line as f64,
                                                    linecol.col_display as f64,
                                                );
                                                
                                                let props = vec![
                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(quote_ident!("name")),
                                                        value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: comp_ident.sym.clone(), raw: None, }))),
                                                    }))),
                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(quote_ident!("id")),
                                                        value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: hash.into(), raw: None, }))),
                                                    }))),
                                                     PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(quote_ident!("file")),
                                                        value: Box::new(Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: self.file_path.clone().into(), raw: None, }))),
                                                    }))),
                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(quote_ident!("line")),
                                                        value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: linecol.line as f64, raw: None, }))),
                                                    }))),
                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(quote_ident!("offset")),
                                                        value: Box::new(Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: linecol.col_display as f64, raw: None, }))),
                                                    }))),
                                                ];

                                                let meta_obj = Expr::Object(ObjectLit {
                                                    span: DUMMY_SP,
                                                    props,
                                                });
    
                                                Stmt::Decl(Decl::Var(Box::new(VarDecl {
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
                                                })))
                                            };
    
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
                                                                arg: Some(self.wrap_in_h_re(expr.clone())),
                                                            }),
                                                        ],
                                                        ctxt: SyntaxContext::empty(),
                                                    })
                                                }
                                            });
                                        },
                                        Expr::Fn(fn_expr) => {
                                            if let Some(body) = &mut fn_expr.function.body {
                                                self.instrument_function_body(body, &comp_ident, fn_expr.function.span);
                                            }
                                            fn_expr.function.visit_mut_children_with(self);
                                        },
                                        _ => {
                                             init_expr.visit_mut_children_with(self);
                                        }
                                    }
                                     self.current_component = prev_component;
                                }
                            }
                        }
                    }
                }
                var_decl.visit_mut_children_with(self);
            },
            _ => {
                n.visit_mut_children_with(self);
            }
        }
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
                            let mut hook_meta_props = vec![
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("id")),
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
                                        key: PropName::Ident(quote_ident!("file")),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: self.file_path.clone().into(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("hook")),
                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: id.sym.clone(),
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("line")),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.line as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("offset")),
                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: linecol.col_display as f64,
                                            raw: None,
                                        }))),
                                    }))),
                                ];

                            if self.include_arguments {
                                let mut args_vec = Vec::new();
                                for arg in &call.args {
                                    if arg.spread.is_some() {
                                        args_vec.push(Some(ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(Expr::Lit(Lit::Str(Str {
                                                span: DUMMY_SP,
                                                value: "...".into(),
                                                raw: None,
                                            }))),
                                        }));
                                        continue;
                                    }

                                    let arg_str = self.cm.span_to_snippet(arg.expr.span()).unwrap_or_else(|_| "<unsupported>".to_string());
                                    
                                    args_vec.push(Some(ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: arg_str.into(),
                                            raw: None,
                                        }))),
                                    }));
                                }

                                hook_meta_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Ident(quote_ident!("arguments")),
                                    value: Box::new(Expr::Array(ArrayLit {
                                        span: DUMMY_SP,
                                        elems: args_vec,
                                    })),
                                }))));
                            }

                            let hook_meta = Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: hook_meta_props,
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
                                            prop: MemberProp::Ident(quote_ident!("s")),
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
                                            prop: MemberProp::Ident(quote_ident!("e")),
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

    fn visit_mut_return_stmt(&mut self, n: &mut ReturnStmt) {
        // First recurse.
        n.visit_mut_children_with(self);

        // Only touch code while we're inside a component.
        if self.current_component.is_none() {
            return;
        }

        // Only wrap `return <expr>;`
        if let Some(arg) = &mut n.arg {
            // Avoid double‑wrapping.
            let already_wrapped = if let Expr::Call(CallExpr {
                callee: Callee::Expr(callee_expr),
                ..
            }) = &**arg {
                if let Expr::Member(MemberExpr {
                    obj,
                    prop: MemberProp::Ident(prop_ident),
                    ..
                }) = &**callee_expr {
                    if let Expr::Ident(obj_ident) = &**obj {
                        obj_ident.sym == *"h" && prop_ident.sym == *"re"
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
            
            if !already_wrapped {
                *arg = self.wrap_in_h_re(arg.clone());
            }
        }
    }
}

pub fn jitter_pass(
    cm: PluginSourceMapProxy,
    filename: String,
    options: Options,
) -> impl VisitMut {
    JitterTransform::new(cm, filename, options)
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
            stripped.to_string()
        } else {
            filename.clone()
        }
    } else {
        filename.clone()
    };

    // Clone SourceMap proxy
    let cm: PluginSourceMapProxy = metadata.source_map.clone();

    let options = match config {
        Config::WithOptions(opts) => opts,
        _ => Options::default(),
    };

    // Apply jitter_pass visitor
    program.visit_mut_with(&mut jitter_pass(
        cm,
        filename,
        options,
    ));

    program
}

