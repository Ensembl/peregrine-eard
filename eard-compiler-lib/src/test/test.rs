use ordered_float::OrderedFloat;
use crate::{ controller::compiler::EardCompiler, controller::compilation::EardCompilation, controller::source::{CombinedSourceSourceBuilder, FixedSourceSource, CombinedSourceSource, ParsePosition, SourceSourceImpl}, libcore::libcore::libcore_sources, model::constants::Constant, test::testutil::sepfmt};
use crate::frontend::parsetree::{PTExpression};
use super::testharness::run_parse_tests;

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
    let mut compiler = EardCompiler::new().expect("couldn't build compiler");
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
fn test_libcore_arith() {
    run_parse_tests(include_str!("testdata/libcore-arith.etf"),true,false);
}

#[test]
fn test_libcore_logic() {
    run_parse_tests(include_str!("testdata/libcore-logic.etf"),true,false);
}

#[test]
fn test_libcore_string() {
    run_parse_tests(include_str!("testdata/libcore-string.etf"),true,false);
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

#[test]
fn test_handle() {
    run_parse_tests(include_str!("testdata/handle.etf"),true,false);
}

#[test]
fn test_entry() {
    run_parse_tests(include_str!("testdata/entry.etf"),false,false);
}

#[test]
fn test_multi_generate() {
    let source = "
        program \"test\" \"test\" 1;

        world code wc1(number) -> number { impl(r1: number) -> r2: number { opcode 901, r2, r1; } }
        world code wc2(number) -> number { impl(r1: number) -> r2: number { opcode 902, r2, r1; } }
        world code wc3(number) -> number { impl(r1: number) -> r2: number { opcode 903, r2, r1; } }
        world code wc4(number) -> number { impl(r1: number) -> r2: number { opcode 904, r2, r1; } }


        version(<=3)    function c(x) { let y = wc1(x); y }
        version(>3 <=8) function c(x) { let y = wc2(x); y }
        version(=9)     function c(x) { let y = wc3(x); y }
        version(>9)     function c(x) { let y = wc4(x); y }
        
        c(3);
    ";
    let mut chosen = vec![];
    for v in 0..12 {
        let mut compiler = EardCompiler::new().expect("bad compiler");
        compiler.set_target_version(v);
        let mut compilation = EardCompilation::new(&compiler).expect("bad compilation");
        let mut soso_builder = CombinedSourceSourceBuilder::new().expect("cannot create soso");
        soso_builder.add_fixed(libcore_sources());
        soso_builder.add_fixed(FixedSourceSource::new_vec(vec![("test",source)]));
        let soso = CombinedSourceSource::new(&soso_builder);
        let position = ParsePosition::root(SourceSourceImpl::new(soso),"included");
        let stmts = compilation.parse(&position,"test",true).expect("cannot parse");
        let stmts = compilation.preprocess(stmts).expect("preprocess");
        let tree = compilation.build(stmts).expect("building");
        let (step,_) = compilation.middleend(&tree).expect("middleend");
        let text = sepfmt(&mut step.iter(),"\n","");
        for line in text.split("\n") {
            if line.contains("opcode") {
                let digits = line.chars().filter(|x| x.is_digit(10) || *x == ',').collect::<String>();
                if let Some(comma) = digits.find(",") {
                    let (opcode,_) = digits.split_at(comma);
                    chosen.push(opcode.parse::<u32>().expect("bad digit parse"));
                }
            }
        }
    }
    assert_eq!(vec![901, 901, 901, 901, 902, 902, 902, 902, 902, 903, 904, 904],chosen);
}
