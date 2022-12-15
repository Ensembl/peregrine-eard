use std::{collections::{HashMap, HashSet}};
use crate::{frontend::{parsetree::{PTStatement, PTExpression}, preprocess::{preprocess}, parser::parse_earp, buildtree::BuildTree}, model::LinearStatement, narrowtyping::{NarrowType, narrow_type}, unbundle::{linearize::linearize, buildunbundle::build_unbundle}, reduce::reduce, broadtyping::broad_type, checking::run_checking};
use crate::model::{OrBundleRepeater};

pub struct EarpCompiler {
    source_loader: Box<dyn Fn(&str) -> Result<String,String>>,
    block_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],(&[String],usize),usize) -> Result<Vec<PTStatement>,String>>>,
    expression_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],usize) -> Result<PTExpression,String>>>
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
            where F: Fn(&[OrBundleRepeater<PTExpression>],(&[String],usize),usize) -> Result<Vec<PTStatement>,String> + 'static {
        self.check_macro_name_unused(name)?;
        self.block_macros.insert(name.to_string(),Box::new(macro_cb));
        Ok(())
    }

    pub fn add_expression_macro<F>(&mut self, name: &str, macro_cb: F) -> Result<(),String>
            where F: Fn(&[OrBundleRepeater<PTExpression>],usize) -> Result<PTExpression,String> + 'static {
        self.check_macro_name_unused(name)?;
        self.expression_macros.insert(name.to_string(),Box::new(macro_cb));
        Ok(())
    }

    pub(crate) fn apply_block_macro(&self, name: &str, args: &[OrBundleRepeater<PTExpression>], pos: (&[String],usize), context: usize) -> Result<Vec<PTStatement>,String> {
        let cb = self.block_macros.get(name).ok_or_else(|| format!("No such block macro \"{:?}\"",name))?;
        cb(args,pos,context)
    }

    pub(crate) fn apply_expression_macro(&self, name: &str, args: &[OrBundleRepeater<PTExpression>], context: usize) -> Result<PTExpression,String> {
        let cb = self.expression_macros.get(name).ok_or_else(|| format!("No such expression macro \"{:?}\"",name))?;
        cb(args,context)
    }
}

pub struct EarpCompilation<'a> {
    pub(crate) compiler: &'a EarpCompiler,
    pub(crate) flags: HashSet<String>,
    context: usize
}

impl<'a> EarpCompilation<'a> {
    pub fn new(compiler: &'a EarpCompiler) -> EarpCompilation {
        EarpCompilation {
            compiler,
            flags: HashSet::new(),
            context: 0
        }
    }

    pub fn set_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string());
    }

    pub(crate) fn parse(&mut self, filename: &[String]) -> Result<Vec<PTStatement>,String> {
        self.context += 1;
        let context = self.context;
        parse_earp(&self.compiler,filename,context)
    }

    pub(crate) fn preprocess(&mut self, parse_tree: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
        preprocess(self,parse_tree)
    }

    pub(crate) fn build(&mut self, input: Vec<PTStatement>) -> Result<BuildTree,String> {
        PTStatement::to_build_tree(input)    
    }

    pub(crate) fn frontend(&mut self, filename: &str) -> Result<BuildTree,String> {
        let stmts = self.parse(&[filename.to_string()])?;
        let stmts = self.preprocess(stmts)?;
        self.build(stmts)
    }

    pub(crate) fn middleend(&mut self, tree: &BuildTree) -> Result<(Vec<LinearStatement>,HashMap<usize,NarrowType>),String> {
        let bundles = build_unbundle(&tree)?;
        let linear = reduce(&linearize(&tree,&bundles)?);
        let (broad,block_indexes) = broad_type(&tree,&linear)?;
        run_checking(&tree,&linear,&block_indexes)?;
        let narrow = narrow_type(&tree,&broad,&block_indexes,&linear)?;
        Ok((linear,narrow))
    }

    pub fn compile(&mut self, filename: &str) -> Result<(Vec<LinearStatement>,HashMap<usize,NarrowType>),String> {
        let tree = self.frontend(filename)?;
        let (stmts,typing) = self.middleend(&tree)?;
        Ok((stmts,typing))
    }
}
