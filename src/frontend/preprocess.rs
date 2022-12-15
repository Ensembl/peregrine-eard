use std::{collections::HashMap, sync::Arc};
use crate::{compiler::{EarpCompiler, EarpCompilation}, model::OrBundleRepeater};
use super::parsetree::{PTTransformer, PTCall, PTExpression, PTStatement, at};

const INFIX_OPERATORS : [(&'static str,&'static str);12] = [
    ("*", "__operator_mul"),
    ("/", "__operator_div"),
    ("+", "__operator_add"),
    ("-", "__operator_sub"),
    (">", "__operator_gt"),
    (">=","__operator_ge"),
    ("<", "__operator_lt"),
    ("<=","__operator_le"),
    ("==","__operator_eq"),
    ("!=","__operator_ne"),
    ("&&","__operator_and"),
    ("||","__operator_or")
];

const PREFIX_OPERATORS : [(&'static str,&'static str);2] = [
    ("-","__operator_minus"),
    ("!","__operator_not")
];

const CTOR_FINITE : &'static str = "__operator_finite";
const CTOR_INFINITE : &'static str = "__operator_infinite";
const CTOR_PUSH : &'static str = "__operator_push";

struct RunMacrosOnce<'a> {
    compiler: &'a EarpCompiler,
    any: bool
}

impl<'a> PTTransformer for RunMacrosOnce<'a> {
    fn call_to_expr(&mut self, call: &PTCall, context: usize) -> Result<Option<PTExpression>,String> {
        if call.is_macro {
            self.any = true;
            Ok(Some(self.compiler.apply_expression_macro(&call.name,&call.args,context)?))
        } else {
            Ok(None)
        }
    }

    fn call_to_block(&mut self, call: &PTCall, pos: (&[String],usize), context: usize) -> Result<Option<Vec<PTStatement>>,String> {
        if call.is_macro {
            self.any = true;
            Ok(Some(self.compiler.apply_block_macro(&call.name,&call.args,pos,context)?))
        } else {
            Ok(None)
        }
    }
}

fn run_macros_once(compiler: &EarpCompiler, block: Vec<PTStatement>) -> Result<(Vec<PTStatement>,bool),String> {
    let mut once = RunMacrosOnce { compiler, any: false };
    let out = PTStatement::transform_list(block,&mut once)?;
    Ok((out,once.any))
}

struct RunIncludeOnce<'a,'b> {
    compilation: &'b mut EarpCompilation<'a>,
    any: bool
}

impl<'a,'b> PTTransformer for RunIncludeOnce<'a,'b> {
    fn include(&mut self, pos: (&[String],usize), path: &str) -> Result<Option<Vec<PTStatement>>,String> {
        self.any = true;
        let mut full_path = pos.0.to_vec();
        full_path.push(path.to_string());
        Ok(Some(self.compilation.parse(&full_path).map_err(|e| {
            at(&e,Some(pos))
        })?))
    }
}

fn run_include_once<'a,'b>(compilation: &'b mut EarpCompilation<'a>, block: Vec<PTStatement>) -> Result<(Vec<PTStatement>,bool),String> {
    let mut once = RunIncludeOnce { compilation, any: false };
    let out = PTStatement::transform_list(block,&mut once)?;
    Ok((out,once.any))
}

struct Phase2Misc<'a,'b> {
    compilation: &'b mut EarpCompilation<'a>,
    prefix: HashMap<String,String>,
    infix: HashMap<String,String>
}

impl<'a,'b> Phase2Misc<'a,'b> {
    fn new(compilation: &'b mut EarpCompilation<'a>) -> Phase2Misc<'a,'b> {
        Phase2Misc { 
            compilation,
            prefix: PREFIX_OPERATORS.iter().map(|(k,v)| (k.to_string(),v.to_string())).collect(),
            infix: INFIX_OPERATORS.iter().map(|(k,v)| (k.to_string(),v.to_string())).collect()
        }
    }
}

impl<'a,'b> PTTransformer for Phase2Misc<'a,'b> {
    fn remove_flags(&mut self, flag: &str) -> Result<bool,String> {
        self.compilation.set_flag(flag);
        Ok(true)
    }

    fn bad_repeater(&mut self, pos: (&[String],usize)) -> Result<(),String> {
        return Err(at("invalid use of repeater (**)",Some(pos)))
    }

    fn replace_infix(&mut self, a: &PTExpression, f: &str, b: &PTExpression) -> Result<Option<PTExpression>,String> {
        if let Some(name) = self.infix.get(f) {
            Ok(Some(PTExpression::Call(PTCall {
                name: name.to_string(),
                args: vec![
                    OrBundleRepeater::Normal(a.clone()),
                    OrBundleRepeater::Normal(b.clone()),
                ],
                is_macro: false
            })))
        } else {
            panic!("missing operator: {:?}",f); // cos grammar and consts should match
        }
    }

    fn replace_prefix(&mut self, f: &str, a: &PTExpression) -> Result<Option<PTExpression>,String> {
        if let Some(name) = self.prefix.get(f) {
            Ok(Some(PTExpression::Call(PTCall {
                name: name.to_string(),
                args: vec![
                    OrBundleRepeater::Normal(a.clone()),
                ],
                is_macro: false
            })))
        } else {
            panic!("missing operator: {:?}",f); // cos grammar and consts should match
        }
    }
}

fn phase2_misc(compilation: &mut EarpCompilation, block: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
    let mut flags = Phase2Misc::new(compilation);
    PTStatement::transform_list(block,&mut flags)
}

pub fn preprocess(compilation: &mut EarpCompilation, mut block: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
    let mut any1 = true;
    let mut any2 = true;
    while any1 || any2 {
        /* no short circuiting */
        (block,any1) = run_macros_once(&compilation.compiler,block)?;
        (block,any2) = run_include_once(compilation,block)?;
    }
    phase2_misc(compilation,block)
}