use std::{collections::{HashMap, HashSet, BTreeMap}, hash::Hash};
use ordered_float::OrderedFloat;
use crate::{compiler::{EarpCompiler}, model::{Variable, Constant, sepfmt, LinearStatement, FullConstant, dump_opers, dump_linear}, unbundle::{buildunbundle::{trace_build_unbundle, build_unbundle}, linearize::{linearize, Allocator}}, frontend::{buildtree::BuildTree, femodel::{OrBundleRepeater, OrBundle}}, middleend::{reduce::reduce, checking::run_checking, broadtyping::broad_type, narrowtyping::narrow_type, culdesac::culdesac, constfold::const_fold}, compilation::EarpCompilation, reorder::reorder, reuse::{test_reuse, reuse}, spill::spill, generate::generate, source::{ParsePosition, FixedSourceSource, SourceSourceImpl, CombinedSourceSource, CombinedSourceSourceBuilder}, libcore::libcore::libcore_sources};
use crate::frontend::parsetree::{PTExpression, PTStatement, PTStatementValue};

fn sort_map<'a,K: PartialEq+Eq+Hash+Ord,V>(h: &'a HashMap<K,V>) -> Vec<(&'a K,&'a V)> {
    let mut keys = h.keys().collect::<Vec<_>>();
    keys.sort();
    keys.iter().map(|k| (*k,h.get(k).unwrap())).collect()
}

pub(crate) fn make_compiler() -> Result<EarpCompiler,String> {
    let mut compiler = EarpCompiler::new()?;
    compiler.add_block_macro("x", |expr,pos,context| {
        let value = match &expr[0] {
            OrBundleRepeater::Normal(x) => x,
            _ => { return Err(format!("expceted expression")) }
        };
        Ok(vec![PTStatement {
            value: PTStatementValue::LetStatement(
                vec![OrBundleRepeater::Normal((Variable{ prefix: None, name: "x".to_string()},vec![]))],
                vec![OrBundle::Normal(value.clone())]
            ),
            position: pos.clone(),
            context
        }])
    })?;
    compiler.add_expression_macro("y",|expr,_| {
        let value = match &expr[0] {
            OrBundleRepeater::Normal(x) => x,
            _ => { return Err(format!("expceted expression")) }
        };
        Ok(PTExpression::Infix(Box::new(value.clone()),"+".to_string(),Box::new(PTExpression::Constant(Constant::Number(OrderedFloat(1.))))))
    })?;
    compiler.add_block_macro("z", |_expr,pos,context| {
        Ok(vec![PTStatement {
            value: PTStatementValue::Include("test2".to_string(),true),
            position: pos.clone(),
            context
        }])
    })?;
    compiler.add_constant_folder("test",|args| {
        args.iter().cloned().collect::<Option<Vec<FullConstant>>>().and_then(|v| {
            match (&v[0],&v[1]) {
                (FullConstant::Atomic(Constant::Number(a)),FullConstant::Atomic(Constant::Number(b))) => {
                    if *a+*b < OrderedFloat(10.) {
                        Some(vec![FullConstant::Atomic(Constant::Number(*a+*b))])
                    } else {
                        None
                    }        
                },
                _ => None
            }
        })
    })?;
    Ok(compiler)
}

pub(crate) fn make_compilation<'a>(compiler: &'a EarpCompiler, libcore: bool, optimise: bool) -> EarpCompilation<'a> {
    let mut comp = EarpCompilation::new(compiler).expect("cannot build compilation");
    comp.set_optimise(optimise);
    if !libcore {
        comp.set_flag("no-libcore");
    }
    comp
}

fn split_on_space(input: &str) -> Vec<String> {
    let mut parts = vec![];
    for part in input.split(" ") {
        let part = part.trim();
        if !part.is_empty() {
            parts.push(part.to_string());
        }
    }
    parts
}

fn process_ws(input: &str, options: &HashSet<String>) -> String {
    let mut no_lines = String::new();
    let input = if options.contains("nolines") {
        for line in input.split("\n") {
            if let Some(split) = line.find(" ") {
                let (_,main) = line.split_at(split);
                no_lines.push_str(main);
            } else {
                no_lines.push_str(line);
            }
            no_lines.push('\n');
        }
        no_lines
    } else {
        input.to_string()
    };
    let input = if options.contains("main") {
        let mut alpha = String::new();
        for line in input.split("\n") {
            if let Some(first) = line.chars().next() {
                if first.is_alphabetic() && !line.contains("define") {
                    alpha.push_str(line);
                    alpha.push('\n');
                }
            } else {
                alpha.push_str(line);
                alpha.push('\n');
            }
        }
        alpha
    } else {
        input.to_string()
    };
    if options.contains("strip") {
        let mut out = String::new();
        for c in input.chars() {
            let skipped_comma = options.contains("skip-commas") && c == ',';
            if !c.is_whitespace() && !skipped_comma {
                out.push(c);
            }
        }
        out    
    } else if options.contains("trim") {
        input.trim().to_string()
    } else {
        input.to_string()
    }
}

fn frontend(compilation: &mut EarpCompilation, processed: &[PTStatement]) -> (BuildTree,Vec<LinearStatement>,Allocator) {
    let tree = compilation.build(processed.to_vec()).expect("build failed");
    let bundles = build_unbundle(&tree).expect("unbundle failed");
    let (linear,next_register) = linearize(&tree,&bundles).expect("linearize failed");
    (tree,reduce(&linear),next_register)
}

pub(super) fn run_parse_tests(data: &str, libcore: bool, optimise: bool) {
    let mut tests = vec![];
    let mut cur_section = "".to_string();
    let mut cur_name = String::new();
    for line in data.lines() {
        let parts = split_on_space(line);
        if parts.len() > 1 && parts[0] == ">>" && parts[1] == "test" {
            tests.push((HashMap::new(),HashMap::new()));
        }
        if let Some((last_sec,last_input)) = tests.last_mut() {
            if parts.len() > 0 && parts[0] == ">>" {
                cur_section = parts[1].to_string();
                if cur_section == "input" {
                    cur_name = parts.get(2).cloned().unwrap_or("test".to_string());
                    last_input.insert(cur_name.clone(),String::new());
                } else {
                    let mut options = HashSet::new();
                    for option in &parts[2..] {
                        options.insert(option.to_string());
                    }                    
                    last_sec.insert(cur_section.clone(),(options,String::new()));
                }
            } else {
                if cur_section == "input" {
                    last_input.get_mut(&cur_name).unwrap().push_str(&format!("{}\n",line));
                } else {
                    last_sec.get_mut(&cur_section).unwrap().1.push_str(&format!("{}\n",line));
                }
            }
        }
    }
    for (sections,inputs) in tests {
        eprintln!("{:?}",inputs);
        let compiler = make_compiler().ok().unwrap();
        let mut compilation = make_compilation(&compiler,libcore,optimise);
        let input = if let Some(x) = inputs.get("test") { x.clone() } else { continue; };
        println!("\n\n\n{}\n",input);
        let mut soso_builder = CombinedSourceSourceBuilder::new().expect("cannot create soso");
        soso_builder.add_fixed(libcore_sources());
        soso_builder.add_fixed(FixedSourceSource::new(inputs.clone()));
        let soso = CombinedSourceSource::new(&soso_builder);
        let position = ParsePosition::root(SourceSourceImpl::new(soso),"included");
        let parse_tree = compilation.parse(&position,"test",true);
        if let Some((parse_options,parse)) = sections.get("parse-fail") {
            match parse_tree {
                Ok(result) => {
                    eprintln!("FAILED ON \n{}\nUnexpected success:\n{:?}",input,result);
                    assert!(false);
                },
                Err(e) => {
                    if !(process_ws(&e,parse_options).contains(&process_ws(&parse,parse_options))) {
                        eprintln!("{:?} does not contain {:?}",e,parse);
                        assert!(false);
                    }
                }
            }
            continue;
        }
        if let Some((parse_options,parse)) = sections.get("parse") {
            match &parse_tree {
                Ok(result) => {
                    let result_str = format!("{:?}",result);
                    assert_eq!(process_ws(&result_str,parse_options),process_ws(&parse,parse_options));
                },
                Err(e) => {
                    eprintln!("FAILED ON \n{}\nUnexpected error:\n{}",input,e);
                    assert!(false);
                }
            }
        }
        let parse_tree = match parse_tree {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{}",e);
                assert!(false);
                unreachable!();
            }
        };
        let processed = compilation.preprocess(parse_tree);
        if let Some((_,flags)) = sections.get("flags") {
            let want_flags = split_on_space(flags).drain(..).collect();
            assert_eq!(&compilation.flags,&want_flags);
        }
        if let Some((parse_options,parse)) = sections.get("preproc") {
            match &processed {
                Ok(result) => {
                    let result_str = format!("{:?}",result);
                    assert_eq!(process_ws(&result_str,parse_options),process_ws(&parse,parse_options));
                },
                Err(e) => {
                    eprintln!("FAILED ON \n{}\nUnexpected error:\n{}",input,e);
                    assert!(false);
                }
            }
        } else if let Some((parse_options,parse)) = sections.get("preproc-fail") {
            match &processed {
                Ok(result) => {
                    eprintln!("FAILED ON \n{}\nUnexpected success:\n{:?}",input,result);
                    assert!(false);
                },
                Err(e) => {
                    if !(process_ws(&e,parse_options).contains(&process_ws(&parse,parse_options))) {
                        eprintln!("{:?} does not contain {:?}",e,parse);
                        assert!(false);
                    }
                }
            }
            continue;
        }
        if let Some((built_options,built)) = sections.get("built") {
            match &processed {
                Ok(preproc) => {
                    let output = compilation.build(preproc.to_vec());
                    match output {
                        Ok(output) => {
                            eprintln!("{:#?}",output);
                            let built_str = format!("{:?}",output);
                            assert_eq!(process_ws(&built_str,built_options),process_ws(&built,built_options));
                        },
                        Err(e) => {
                            eprintln!("FAILED ON \n{}\nUnexpected error:\n{}",input,e);
                            assert!(false);                                    
                        }
                    }
                },
                Err(e) => {
                    eprintln!("prprocessing failed but expected build: {}",e);
                    assert!(false);
                }
            }
        }
        if let Some((built_options,built_fail)) = sections.get("built-fail") {
            let processed = compilation.build(processed.clone().expect("processing failed")).err().expect("build unexpectedly succeeded");
            println!("{}\n",processed);
            assert_eq!(process_ws(built_fail,built_options),process_ws(&processed,built_options));
            continue;
        }
        if let Some((unbundle_options,unbundle_correct)) = sections.get("unbundle") {
            let processed = compilation.build(processed.clone().expect("processing failed")).expect("build failed");
            let (transits,trace) = trace_build_unbundle(&processed).expect("unbundle failed");
            print!("{}",trace.join("\n"));
            let mut keys = transits.keys();
            keys.sort();
            let mut lines = vec![];
            for (stack,position) in keys {
                let mut values = transits.get(&stack,&position).unwrap().iter().cloned().collect::<Vec<_>>();
                values.sort();
                let key_path = stack.iter().map(|c|{
                    c.to_string()
                }).collect::<Vec<_>>();
                lines.push(format!("{}/{:?}: {}",key_path.join(","),position,values.join(" ")));
            }
            let unbundle_got = lines.join("\n");
            assert_eq!(process_ws(&unbundle_got,unbundle_options),process_ws(unbundle_correct,unbundle_options));
            println!("{}\n",lines.join("\n"));
            if let Some((trace_options,trace_expected)) = sections.get("trace") {
                assert_eq!(process_ws(&trace.join("\n"),trace_options),process_ws(&trace_expected,trace_options));
            }
        }
        if let Some((unbundle_options,unbundle_err)) = sections.get("unbundle-fail") {
            let processed = compilation.build(processed.expect("processing failed")).expect("build failed");
            let error = trace_build_unbundle(&processed).err().expect("unbundle unexpectedly succeeded");
            assert_eq!(process_ws(&error,unbundle_options),process_ws(&unbundle_err,unbundle_options));
            continue;
        }
        if let Some((linearized_options,linearized_correct)) = sections.get("linearize") {
            let tree = compilation.build(processed.clone().expect("processing failed")).expect("build failed");
            let bundles = build_unbundle(&tree).expect("unbundle failed");
            let (mut linear,_) = linearize(&tree,&bundles).expect("linearize failed");
            if linearized_options.contains("reduce") {
                linear = reduce(&linear);
            }
            println!("{}",dump_linear(&linear));
            assert_eq!(process_ws(&dump_linear(&linear),linearized_options),process_ws(linearized_correct,linearized_options));
        }
        if let Some((linearized_options,linearized_correct)) = sections.get("linearize-fail") {
            let tree = compilation.build(processed.expect("processing failed")).expect("build failed");
            let bundles = build_unbundle(&tree).expect("unbundle failed");
            let error = linearize(&tree,&bundles).err().expect("linearize unexpectedly succeeded");
            println!("{}",error);
            assert_eq!(process_ws(&error,linearized_options),process_ws(linearized_correct,linearized_options));
            continue;
        }
        if let Some((broad_options,broad_correct)) = sections.get("broad") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,_) = frontend(&mut compilation,&processed);
            println!("{}",dump_linear(&linear));
            let (typing,_) = broad_type(&tree,&linear).expect("typing failed");
            let mut report = BTreeMap::new();
            for (reg,broad) in sort_map(&typing) {
                report.entry(format!("{:?}",broad)).or_insert(vec![]).push(reg.to_string());
            }
            let report = report.iter().map(|(k,v)| format!("{}: {}",k,v.join(", "))).collect::<Vec<_>>().join("\n");
            println!("{}",report);
            assert_eq!(process_ws(&report,broad_options),process_ws(broad_correct,broad_options));
        }
        if let Some((broad_options,broad_correct)) = sections.get("broad-fail") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,_) = frontend(&mut compilation,&processed);
            println!("{}",dump_linear(&linear));
            let got = broad_type(&tree,&linear).err().expect("typing failed");
            println!("{}",got);
            assert_eq!(process_ws(&got,broad_options),process_ws(broad_correct,broad_options));
        }
        if let Some((checking_options,checking_correct)) = sections.get("checking") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            println!("{}",dump_linear(&linear));
            assert_eq!(process_ws(&dump_linear(&linear),checking_options),process_ws(checking_correct,checking_options));
        }
        if let Some((checking_options,checking_expected)) = sections.get("checking-fail") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("typing failed");
            let error = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).err().expect("checking unexpectedly succeeded");
            assert_eq!(process_ws(&error,checking_options),process_ws(checking_expected,checking_options));
        }
        if let Some((narrow_options,narrow_correct)) = sections.get("narrow") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut report = BTreeMap::new();
            for (reg,narrow) in sort_map(&narrow) {
                report.entry(format!("{:?}",narrow)).or_insert(vec![]).push(reg.to_string());
            }
            let report = report.iter().map(|(k,v)| format!("{}: {}",k,v.join(", "))).collect::<Vec<_>>().join("\n");
            println!("{}",report);
            assert_eq!(process_ws(&report,narrow_options),process_ws(narrow_correct,narrow_options));
        }
        if let Some((narrow_options,narrow_correct)) = sections.get("narrow-fail") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let error = narrow_type(&tree,&broad,&block_indexes, &linear).err().expect("narrow unespectedly succeeded");
            println!("{}",error);
            assert_eq!(process_ws(&error,narrow_options),process_ws(narrow_correct,narrow_options));
        }
        if let Some((constfold_options,constfold_correct)) = sections.get("constfold") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let _narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut opers = const_fold(&compilation,&tree,&block_indexes,&linear);
            if constfold_options.contains("culdesac") {
                opers = culdesac(&tree,&block_indexes,&opers);
            }
            println!("{}",dump_opers(&opers));
            assert_eq!(process_ws(&dump_opers(&opers),constfold_options),process_ws(constfold_correct,constfold_options));
        }
        if let Some((reuse_options,reuse_correct)) = sections.get("reuse") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let _narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut opers = const_fold(&compilation,&tree,&block_indexes,&linear);
            opers = culdesac(&tree,&block_indexes,&opers);
            let (new_opers, knowns) = test_reuse(&tree,&block_indexes,&opers).expect("reuse failed");
            println!("FROM:\n {}\n\n",dump_opers(&opers));
            println!("TO:\n{}\n",dump_opers(&new_opers));
            assert_eq!(process_ws(&dump_opers(&new_opers),reuse_options),process_ws(reuse_correct,reuse_options));
            if let Some((knowns_options,knowns_correct)) = sections.get("reuse-known") {
                let knowns = knowns.iter().collect::<BTreeMap<_,_>>();
                let knowns = knowns.iter().map(|(k,v)| format!("{}: {:?}",k,v)).collect::<Vec<_>>();
                println!("{}",knowns.join("\n"));
                assert_eq!(process_ws(&knowns.join("\n"),knowns_options),process_ws(knowns_correct,knowns_options));
            }
        }
        if let Some((spill_options,spill_correct)) = sections.get("spill") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register, &mut broad).expect("checking unexpectedly failed");
            let mut narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut opers = const_fold(&compilation,&tree,&block_indexes,&linear);
            opers = culdesac(&tree,&block_indexes,&opers);
            opers = reuse(&tree,&block_indexes,&opers).expect("reuse failed");
            opers = spill(next_register,&opers, &mut narrow);
            println!("spilled:\n{}",dump_opers(&opers));
            assert_eq!(process_ws(&dump_opers(&opers),spill_options),process_ws(spill_correct,spill_options));
        }
        if let Some((reorder_options,reorder_correct)) = sections.get("reordered") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let mut narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut opers = const_fold(&compilation,&tree,&block_indexes,&linear);
            opers = culdesac(&tree,&block_indexes,&opers);
            opers = reuse(&tree,&block_indexes,&opers).expect("reuse failed");
            opers = spill(next_register,&opers,&mut narrow);
            opers = reorder(&tree,&block_indexes,&opers).expect("reorder failed");
            println!("reordered:\n{}",dump_opers(&opers));
            assert_eq!(process_ws(&dump_opers(&opers),reorder_options),process_ws(reorder_correct,reorder_options));
        }
        if let Some((generate_options,generate_correct)) = sections.get("generate") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear,mut next_register) = frontend(&mut compilation,&processed);
            let (mut broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            let linear = run_checking(&tree,&linear,&block_indexes,&mut next_register,&mut broad).expect("checking unexpectedly failed");
            let mut narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut opers = const_fold(&compilation,&tree,&block_indexes,&linear);
            opers = culdesac(&tree,&block_indexes,&opers);
            opers = reuse(&tree,&block_indexes,&opers).expect("reuse failed");
            opers = spill(next_register,&opers,&mut narrow);
            opers = reorder(&tree,&block_indexes,&opers).expect("reorder failed");
            let steps = generate(&tree,&block_indexes,&narrow,&opers).expect("generate failed");
            println!("steps:\n{}",sepfmt(&mut steps.iter(),"\n",""));
            assert_eq!(process_ws(&sepfmt(&mut steps.iter(),"\n",""),generate_options),process_ws(generate_correct,generate_options));
        }
    }
}
