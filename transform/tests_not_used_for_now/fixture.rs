use std::path::PathBuf;

use react_jitter::{Options, Config, create_pass};
use swc_common::Mark;
use swc_ecma_parser::{EsSyntax, Syntax};
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_testing::{test_fixture, FixtureTestConfig};

fn syntax(_: &PathBuf) -> Syntax {
    Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
    })
}

#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");

    test_fixture(
        syntax(&input),
        &|t| {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            let options = Options {
                ..Default::default()
            };

            create_pass(
                resolver(unresolved_mark, top_level_mark, false),
                ::react_jitter::react_jitter(
                    t.cm.clone(),
                    Config::WithOptions(options),
                    input.to_string_lossy().to_string(),
                )
            )
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}
