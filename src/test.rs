use crate::{testharness::{run_parse_tests}, compiler::EarpCompiler, parsetree::{PTExpression}, model::Constant};

#[test]
fn test_parse_smoke() {
    run_parse_tests(include_str!("testdata/parser-smoke.etf"));
}

#[test]
fn test_parse_constants() {
    run_parse_tests(include_str!("testdata/parser-constants.etf"));
}

#[test]
fn test_parse_general() {
    run_parse_tests(include_str!("testdata/parser-general.etf"));
}

#[test]
fn test_parse_expressions() {
    run_parse_tests(include_str!("testdata/parser-expressions.etf"));
}

#[test]
fn test_parse_funcproc() {
    run_parse_tests(include_str!("testdata/parser-funcproc.etf"));
}

#[test]
fn test_parse_code() {
    run_parse_tests(include_str!("testdata/parser-code.etf"));
}

#[test]
fn test_preprocess() {
    run_parse_tests(include_str!("testdata/preprocess.etf"));
}

#[test]
fn test_macro_clash() {
    let mut compiler = EarpCompiler::new();
    assert!(compiler.add_block_macro("x", |expr,pos,_| { Ok(vec![])}).is_ok());
    assert!(compiler.add_block_macro("y", |expr,pos,_| { Ok(vec![])}).is_ok());
    assert!(compiler.add_block_macro("x", |expr,pos,_| { Ok(vec![])}).is_err());
    assert!(compiler.add_expression_macro("x", |expr,_| { Ok(
        PTExpression::Constant(Constant::Number(0.))
    )}).is_err());
}

#[test]
fn test_buildtree_smoke() {
    run_parse_tests(include_str!("testdata/buildtree-smoke.etf"));
}

#[test]
fn test_bundle_smoke() {
    run_parse_tests(include_str!("testdata/bundle-smoke.etf"));
}
