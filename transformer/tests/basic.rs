use swc_core::common::{sync::Lrc, FileName, SourceMap};
use swc_core::ecma::codegen::{text_writer::JsWriter, Emitter, Node};
use swc_core::ecma::parser::{Parser, StringInput, Syntax, TsConfig};
use transformer::transform;
use xxhash_rust::xxh3::xxh3_64;

fn apply(file: &str, src: &str) -> String {
    let cm: Lrc<SourceMap> = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Custom(file.into()), src.into());
    let mut parser = Parser::new(
        Syntax::Typescript(TsConfig {
            tsx: true,
            ..Default::default()
        }),
        StringInput::from(&*fm),
        None,
    );
    let program = parser.parse_program().unwrap();
    let program = transform(program, file, Some(&cm));
    let mut buf = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
        };
        program.emit_with(&mut emitter).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

fn id(file: &str, line: i64, offset: i64) -> String {
    format!(
        "{:08x}",
        xxh3_64(format!("{}:{}:{}", file, line, offset).as_bytes())
    )
}

#[test]
fn single_hook() {
    let src = "export function Foo() {\n  const x = useBar();\n  return <div>{x}</div>;\n}";
    let id0 = id("Foo.tsx", 2, 13);
    let expected = format!(
        "import {{ useJitterScope }} from \"react-jitter\";\nexport function Foo() {{\n    const h = useJitterScope(\"Foo\");\n    const x = h.s(), h.e(\"{}\", useBar(), {{\n        id: \"{}\",\n        file: \"Foo.tsx\",\n        hook: \"useBar\",\n        line: 2,\n        offset: 13\n    }});\n    return <div>{{x}}</div>;\n}}",
        id0,
        id0
    );
    let out = apply("Foo.tsx", src);
    assert_eq!(out.trim(), expected);
}

#[test]
fn multiple_hooks() {
    let src = "export function Multi() {\n  const a = useA();\n  const b = useB();\n  return <>{a}{b}</>;\n}";
    let id_a = id("Multi.tsx", 2, 13);
    let id_b = id("Multi.tsx", 3, 13);
    let expected = format!(
        "import {{ useJitterScope }} from \"react-jitter\";\nexport function Multi() {{\n    const h = useJitterScope(\"Multi\");\n    const a = h.s(), h.e(\"{}\", useA(), {{\n        id: \"{}\",\n        file: \"Multi.tsx\",\n        hook: \"useA\",\n        line: 2,\n        offset: 13\n    }});\n    const b = h.s(), h.e(\"{}\", useB(), {{\n        id: \"{}\",\n        file: \"Multi.tsx\",\n        hook: \"useB\",\n        line: 3,\n        offset: 13\n    }});\n    return <>{{a}}{{b}}</>;\n}}",
        id_a,
        id_a,
        id_b,
        id_b
    );
    let out = apply("Multi.tsx", &src);
    assert_eq!(out.trim(), expected);
}

#[test]
fn arrow_component() {
    let src = "const A = () => {\n  const v = useValue();\n  return <span>{v}</span>;\n};\nexport default A;";
    let id0 = id("Arrow.tsx", 2, 13);
    let expected = format!(
        "import {{ useJitterScope }} from \"react-jitter\";\nconst A = ()=>{{\n    const h = useJitterScope(\"A\");\n    const v = h.s(), h.e(\"{}\", useValue(), {{\n        id: \"{}\",\n        file: \"Arrow.tsx\",\n        hook: \"useValue\",\n        line: 2,\n        offset: 13\n    }});\n    return <span>{{v}}</span>;\n}};\nexport default A;",
        id0,
        id0
    );
    let out = apply("Arrow.tsx", src);
    assert_eq!(out.trim(), expected);
}

#[test]
fn import_injection_missing() {
    let src = "export function Foo() {\n  const x = useBar();\n}";
    let id0 = id("Inject.tsx", 2, 13);
    let expected = format!(
        "import {{ useJitterScope }} from \"react-jitter\";\nexport function Foo() {{\n    const h = useJitterScope(\"Foo\");\n    const x = h.s(), h.e(\"{}\", useBar(), {{\n        id: \"{}\",\n        file: \"Inject.tsx\",\n        hook: \"useBar\",\n        line: 2,\n        offset: 13\n    }});\n}}",
        id0,
        id0
    );
    let out = apply("Inject.tsx", src);
    assert_eq!(out.trim(), expected);
}

#[test]
fn import_injection_exists() {
    let src = "import { useJitterScope } from 'react-jitter';\nexport function Foo() {\n  const x = useBar();\n}";
    let id0 = id("Exists.tsx", 3, 13);
    let expected = format!(
        "import {{ useJitterScope }} from 'react-jitter';\nexport function Foo() {{\n    const h = useJitterScope(\"Foo\");\n    const x = h.s(), h.e(\"{}\", useBar(), {{\n        id: \"{}\",\n        file: \"Exists.tsx\",\n        hook: \"useBar\",\n        line: 3,\n        offset: 13\n    }});\n}}",
        id0,
        id0
    );
    let out = apply("Exists.tsx", src);
    assert_eq!(out.trim(), expected);
}

#[test]
fn hook_in_if() {
    let src = "export function Cond({ flag }: { flag: boolean }) {\n  if (flag) {\n    const v = useV();\n    return <p>{v}</p>;\n  }\n  return null;\n}";
    let id0 = id("Cond.tsx", 3, 15);
    let expected = format!(
        "import {{ useJitterScope }} from \"react-jitter\";\nexport function Cond({{ flag }}: {{\n    flag: boolean;\n}}) {{\n    const h = useJitterScope(\"Cond\");\n    if (flag) {{\n        const v = h.s(), h.e(\"{}\", useV(), {{\n            id: \"{}\",\n            file: \"Cond.tsx\",\n            hook: \"useV\",\n            line: 3,\n            offset: 15\n        }});\n        return <p>{{v}}</p>;\n    }}\n    return null;\n}}",
        id0,
        id0
    );
    let out = apply("Cond.tsx", src);
    assert_eq!(out.trim(), expected);
}
