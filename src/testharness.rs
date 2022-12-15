use std::{collections::{HashMap, HashSet, BTreeMap}, sync::Arc, hash::Hash};
use crate::{compiler::{EarpCompiler}, model::{Variable, Constant, OrBundle, OrBundleRepeater, sepfmt, LinearStatement}, unbundle::{buildunbundle::{trace_build_unbundle, build_unbundle}, linearize::linearize}, frontend::buildtree::BuildTree, middleend::{reduce::reduce, checking::run_checking, broadtyping::broad_type, narrowtyping::narrow_type}, compilation::EarpCompilation};
use crate::frontend::parsetree::{PTExpression, PTStatement, PTStatementValue};

fn source_loader(sources: HashMap<String,String>) -> impl Fn(&str) -> Result<String,String> {
    move |key| sources.get(key).cloned().ok_or_else(|| "Not found".to_string())
}

fn sort_map<'a,K: PartialEq+Eq+Hash+Ord,V>(h: &'a HashMap<K,V>) -> Vec<(&'a K,&'a V)> {
    let mut keys = h.keys().collect::<Vec<_>>();
    keys.sort();
    keys.iter().map(|k| (*k,h.get(k).unwrap())).collect()
}

pub(crate) fn make_compiler(sources: HashMap<String,String>) -> Result<EarpCompiler,String> {
    let mut compiler = EarpCompiler::new();
    compiler.set_source_loader(source_loader(sources));
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
            file: Arc::new(pos.0.to_vec()),
            line_no: pos.1,
            context
        }])
    })?;
    compiler.add_expression_macro("y",|expr,_| {
        let value = match &expr[0] {
            OrBundleRepeater::Normal(x) => x,
            _ => { return Err(format!("expceted expression")) }
        };
        Ok(PTExpression::Infix(Box::new(value.clone()),"+".to_string(),Box::new(PTExpression::Constant(Constant::Number(1.)))))
    })?;
    compiler.add_block_macro("z", |_expr,pos,context| {
        Ok(vec![PTStatement {
            value: PTStatementValue::Include("test2".to_string()),
            file: Arc::new(pos.0.to_vec()),
            line_no: pos.1,
            context
        }])
    })?;
    Ok(compiler)
}

pub(crate) fn make_compilation<'a>(compiler: &'a EarpCompiler) -> EarpCompilation<'a> {
    EarpCompilation::new(compiler)
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

fn frontend(compilation: &mut EarpCompilation, processed: &[PTStatement]) -> (BuildTree,Vec<LinearStatement>) {
    let tree = compilation.build(processed.to_vec()).expect("build failed");
    let bundles = build_unbundle(&tree).expect("unbundle failed");
    let linear = linearize(&tree,&bundles).expect("linearize failed");
    (tree,reduce(&linear))
}

pub(super) fn run_parse_tests(data: &str) {
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
        let compiler = make_compiler(inputs.clone()).ok().unwrap();
        let mut compilation = make_compilation(&compiler);
        let input = if let Some(x) = inputs.get("test") { x.clone() } else { continue; };
        println!("\n\n\n{}\n",input);
        let parse_tree = compilation.parse(&Arc::new(vec!["test".to_string()]));
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
            let mut linear = linearize(&tree,&bundles).expect("linearize failed");
            if linearized_options.contains("reduce") {
                linear = reduce(&linear);
            }
            let linear = linear.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>();
            println!("{}",linear.join("\n"));
            assert_eq!(process_ws(&linear.join("\n"),linearized_options),process_ws(linearized_correct,linearized_options));
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
            let (tree,linear) = frontend(&mut compilation,&processed);
            println!("{}",sepfmt(&mut linear.iter(),"\n",""));
            let (typing,_) = broad_type(&tree,&linear).expect("typing failed");
            let mut report = BTreeMap::new();
            for (reg,broad) in sort_map(&typing) {
                report.entry(format!("{:?}",broad)).or_insert(vec![]).push(reg.to_string());
            }
            let report = report.iter().map(|(k,v)| format!("{}: {}",k,v.join(", "))).collect::<Vec<_>>().join("\n");
            println!("{}",report);
            assert_eq!(process_ws(&report,broad_options),process_ws(broad_correct,broad_options));
        }
        if let Some((_checking_options,_checking_correct)) = sections.get("checking") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear) = frontend(&mut compilation,&processed);
            let (_typing,block_indexes) = broad_type(&tree,&linear).expect("typing failed");
            run_checking(&tree,&linear,&block_indexes).expect("checking unexpectedly failed");
        }
        if let Some((checking_options,checking_expected)) = sections.get("checking-fail") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear) = frontend(&mut compilation,&processed);
            let (_typing,block_indexes) = broad_type(&tree,&linear).expect("typing failed");
            let error = run_checking(&tree,&linear,&block_indexes).err().expect("checking unexpectedly succeeded");
            assert_eq!(process_ws(&error,checking_options),process_ws(checking_expected,checking_options));
        }
        if let Some((narrow_options,narrow_correct)) = sections.get("narrow") {
            let processed = processed.clone().expect("processing failed");
            let (tree,linear) = frontend(&mut compilation,&processed);
            let (broad,block_indexes) = broad_type(&tree,&linear).expect("broad typing failed");
            run_checking(&tree,&linear,&block_indexes).expect("checking unexpectedly failed");
            let narrow = narrow_type(&tree,&broad,&block_indexes, &linear).expect("narrow typing failed");
            let mut report = BTreeMap::new();
            for (reg,narrow) in sort_map(&narrow) {
                report.entry(format!("{:?}",narrow)).or_insert(vec![]).push(reg.to_string());
            }
            let report = report.iter().map(|(k,v)| format!("{}: {}",k,v.join(", "))).collect::<Vec<_>>().join("\n");
            println!("{}",report);
            assert_eq!(process_ws(&report,narrow_options),process_ws(narrow_correct,narrow_options));
        }
    }
}
