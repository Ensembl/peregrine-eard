use std::{collections::{HashMap, HashSet}, sync::Arc};
use crate::{parsetree::{PTCallArg, PTStatement, PTExpression}, preprocess::{preprocess}, parser::parse_earp, buildtree::BuildTree};

pub struct EarpCompiler {
    source_loader: Box<dyn Fn(&str) -> Result<String,String>>,
    block_macros: HashMap<String,Box<dyn Fn(&[PTCallArg],(&[String],usize)) -> Result<Vec<PTStatement>,String>>>,
    expression_macros: HashMap<String,Box<dyn Fn(&[PTCallArg]) -> Result<PTExpression,String>>>
}

impl EarpCompiler {
    pub fn new() -> EarpCompiler {
        EarpCompiler {
            source_loader: Box::new(|_| { Err("No source loader set".to_string()) }),
            block_macros: HashMap::new(),
            expression_macros: HashMap::new()
        }
    }

    pub fn set_source_loader<F>(&mut self, cb: F)
            where F: Fn(&str) -> Result<String,String> + 'static {
        self.source_loader = Box::new(cb);
    }

    pub fn load_source(&self, path: &str) -> Result<String,String> {
        (self.source_loader)(path).map_err(|e| format!("Error loading '{}': {}",path,e))
    }

    fn check_macro_name_unused(&self, name: &str) -> Result<(),String> {
        if self.block_macros.contains_key(name) || self.expression_macros.contains_key(name) {
            Err(format!("Duplicate macro definition '{}'",name))
        } else {
            Ok(())
        }
    }

    pub fn add_block_macro<F>(&mut self, name: &str, macro_cb: F) -> Result<(),String>
            where F: Fn(&[PTCallArg],(&[String],usize)) -> Result<Vec<PTStatement>,String> + 'static {
        self.check_macro_name_unused(name)?;
        self.block_macros.insert(name.to_string(),Box::new(macro_cb));
        Ok(())
    }

    pub fn add_expression_macro<F>(&mut self, name: &str, macro_cb: F) -> Result<(),String>
            where F: Fn(&[PTCallArg]) -> Result<PTExpression,String> + 'static {
        self.check_macro_name_unused(name)?;
        self.expression_macros.insert(name.to_string(),Box::new(macro_cb));
        Ok(())
    }

    pub(crate) fn apply_block_macro(&self, name: &str, args: &[PTCallArg], pos: (&[String],usize)) -> Result<Vec<PTStatement>,String> {
        let cb = self.block_macros.get(name).ok_or_else(|| format!("No such block macro \"{:?}\"",name))?;
        cb(args,pos)
    }

    pub(crate) fn apply_expression_macro(&self, name: &str, args: &[PTCallArg]) -> Result<PTExpression,String> {
        let cb = self.expression_macros.get(name).ok_or_else(|| format!("No such expression macro \"{:?}\"",name))?;
        cb(args)
    }
}

pub struct EarpCompilation<'a> {
    pub(crate) compiler: &'a EarpCompiler,
    pub(crate) flags: HashSet<String>
}

impl<'a> EarpCompilation<'a> {
    pub fn new(compiler: &'a EarpCompiler) -> EarpCompilation {
        EarpCompilation {
            compiler,
            flags: HashSet::new()
        }
    }

    pub fn set_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string());
    }

    pub fn parse(&mut self, filename: &[String]) -> Result<Vec<PTStatement>,String> {
        parse_earp(&self.compiler,filename)
    }

    pub fn preprocess(&mut self, parse_tree: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
        preprocess(self,parse_tree)
    }

    pub fn build(&mut self, input: Vec<PTStatement>) -> Result<BuildTree,String> {
        PTStatement::to_build_tree(input)    
    }
}
