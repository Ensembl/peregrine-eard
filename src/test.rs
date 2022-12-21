use ordered_float::OrderedFloat;

use crate::{testharness::{run_parse_tests}, compiler::EarpCompiler, model::Constant};
use crate::frontend::parsetree::{PTExpression};

#[test]
fn test_parse_smoke() {
    run_parse_tests(include_str!("testdata/parser-smoke.etf"),false,false);
}

#[test]
fn test_parse_constants() {
    run_parse_tests(include_str!("testdata/parser-constants.etf"),false,false);
}

#[test]
fn test_parse_general() {
    run_parse_tests(include_str!("testdata/parser-general.etf"),false,false);
}

#[test]
fn test_parse_expressions() {
    run_parse_tests(include_str!("testdata/parser-expressions.etf"),false,false);
}

#[test]
fn test_parse_funcproc() {
    run_parse_tests(include_str!("testdata/parser-funcproc.etf"),false,false);
}

#[test]
fn test_parse_code() {
    run_parse_tests(include_str!("testdata/parser-code.etf"),false,false);
}

#[test]
fn test_preprocess() {
    run_parse_tests(include_str!("testdata/preprocess.etf"),false,false);
}

#[test]
fn test_macro_clash() {
    let mut compiler = EarpCompiler::new().expect("couldn't build compiler");
    assert!(compiler.add_block_macro("x", |_,_,_| { Ok(vec![])}).is_ok());
    assert!(compiler.add_block_macro("y", |_,_,_| { Ok(vec![])}).is_ok());
    assert!(compiler.add_block_macro("x", |_,_,_| { Ok(vec![])}).is_err());
    assert!(compiler.add_expression_macro("x", |_,_| { Ok(
        PTExpression::Constant(Constant::Number(OrderedFloat(0.)))
    )}).is_err());
}

#[test]
fn test_buildtree_smoke() {
    run_parse_tests(include_str!("testdata/buildtree-smoke.etf"),false,false);
}

#[test]
fn test_bundle_smoke() {
    run_parse_tests(include_str!("testdata/bundle-smoke.etf"),false,false);
}

#[test]
fn test_bundle_inch() {
    run_parse_tests(include_str!("testdata/bundle-inch.etf"),false,false);
}

#[test]
fn test_repeat_smoke() {
    run_parse_tests(include_str!("testdata/repeat-smoke.etf"),false,false);
}

#[test]
fn test_linearize_smoke() {
    run_parse_tests(include_str!("testdata/linearize-smoke.etf"),false,false);
}

#[test]
fn test_linearize_fail_smoke() {
    run_parse_tests(include_str!("testdata/linearize-fail.etf"),false,false);
}

#[test]
fn test_linearize_code_smoke() {
    run_parse_tests(include_str!("testdata/linearize-code.etf"),false,false);
}

#[test]
fn test_capture_smoke() {
    run_parse_tests(include_str!("testdata/linearize-capture.etf"),false,false);
}

#[test]
fn test_types() {
    run_parse_tests(include_str!("testdata/types.etf"),true,false);
}

#[test]
fn test_const_fold() {
    run_parse_tests(include_str!("testdata/constfold.etf"),true,false);
}

#[test]
fn test_libcore() {
    run_parse_tests(include_str!("testdata/libcore.etf"),true,false);
}

#[test]
fn test_reorder() {
    run_parse_tests(include_str!("testdata/reorder.etf"),true,false);
}

#[test]
fn test_generate() {
    run_parse_tests(include_str!("testdata/generate.etf"),true,false);
}

#[test]
fn test_checking() {
    run_parse_tests(include_str!("testdata/checking.etf"),true,false);
}

#[test]
fn test_checking_opt() {
    run_parse_tests(include_str!("testdata/checking-opt.etf"),true,true);
}
