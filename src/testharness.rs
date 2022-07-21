use std::{collections::{HashMap, HashSet}, sync::Arc};

use crate::{compiler::{EarpCompiler, EarpCompilation}, parsetree::{PTExpression, PTStatement, PTLetAssign, PTCallArg, PTVariable, PTConstant, PTStatementValue}};

fn source_loader(sources: HashMap<String,String>) -> impl Fn(&str) -> Result<String,String> {
    move |key| sources.get(key).cloned().ok_or_else(|| "Not found".to_string())
}

pub(crate) fn make_compiler(sources: HashMap<String,String>) -> Result<EarpCompiler,String> {
    let mut compiler = EarpCompiler::new();
    compiler.set_source_loader(source_loader(sources));
    compiler.add_block_macro("x", |expr,pos| {
        let value = match &expr[0] {
            PTCallArg::Expression(x) => x,
            _ => { return Err(format!("expceted expression")) }
        };
        Ok(vec![PTStatement {
            value: PTStatementValue::LetStatement(
                PTLetAssign::Variable(vec![(PTVariable{ prefix: None, name: "x".to_string()},vec![])]),
                value.clone()
            ),
            file: Arc::new(pos.0.to_vec()),
            line_no: pos.1
        }])
    })?;
    compiler.add_expression_macro("y",|expr| {
        let value = match &expr[0] {
            PTCallArg::Expression(x) => x,
            _ => { return Err(format!("expceted expression")) }
        };
        Ok(PTExpression::Infix(Box::new(value.clone()),"+".to_string(),Box::new(PTExpression::Constant(PTConstant::Number(1.)))))
    })?;
    compiler.add_block_macro("z", |_expr,pos| {
        Ok(vec![PTStatement {
            value: PTStatementValue::Include("test2".to_string()),
            file: Arc::new(pos.0.to_vec()),
            line_no: pos.1
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
            if !c.is_whitespace() {
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
            match processed {
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
            match processed {
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
    }
}    
