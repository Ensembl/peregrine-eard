use std::collections::{HashSet, HashMap};
use crate::{compiler::EarpCompiler, frontend::{parsetree::PTStatement, buildtree::BuildTree, preprocess::preprocess, parser::{parse_earp, parse_libcore}}, model::{Operation}, unbundle::{buildunbundle::build_unbundle, linearize::linearize}, middleend::{reduce::reduce, checking::run_checking, broadtyping::broad_type, narrowtyping::{narrow_type, NarrowType}, constfold::const_fold, culdesac::culdesac}};

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

    pub(crate) fn compiler(&self) -> &EarpCompiler { &self.compiler }

    pub fn set_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string());
    }

    pub(crate) fn parse_part(&mut self, filename: &[String]) -> Result<Vec<PTStatement>,String> {
        self.context += 1;
        let context = self.context;
        parse_earp(&self.compiler,filename,context)
    }

    fn parse_libcore(&mut self) -> Result<Vec<PTStatement>,String> {
        if self.flags.contains("no-libcore") { return Ok(vec![]); }
        self.context += 1;
        let context = self.context;
        parse_libcore(context)
    }

    pub(crate) fn parse(&mut self, filename: &[String]) -> Result<Vec<PTStatement>,String> {
        let mut out = self.parse_libcore()?;
        out.append(&mut self.parse_part(filename)?);
        Ok(out)
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

    pub(crate) fn middleend(&mut self, tree: &BuildTree) -> Result<(Vec<Operation>,HashMap<usize,NarrowType>),String> {
        let bundles = build_unbundle(&tree)?;
        let linear = reduce(&linearize(&tree,&bundles)?);
        let (broad,block_indexes) = broad_type(&tree,&linear)?;
        run_checking(&tree,&linear,&block_indexes)?;
        let narrow = narrow_type(&tree,&broad,&block_indexes,&linear)?;
        let opers = const_fold(&self,tree,&block_indexes,&linear);
        let opers = culdesac(tree,&block_indexes,&opers);
        Ok((opers,narrow))
    }

    pub fn compile(&mut self, filename: &str) -> Result<(Vec<Operation>,HashMap<usize,NarrowType>),String> {
        let tree = self.frontend(filename)?;
        let (opers,typing) = self.middleend(&tree)?;
        Ok((opers,typing))
    }
}
