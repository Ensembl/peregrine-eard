use std::collections::{HashSet};
use crate::{compiler::EarpCompiler, frontend::{parsetree::{PTStatement, PTStatementValue}, buildtree::BuildTree, preprocess::preprocess, parser::{parse_earp}}, model::{Step}, unbundle::{buildunbundle::build_unbundle, linearize::linearize}, middleend::{reduce::reduce, checking::run_checking, broadtyping::broad_type, narrowtyping::narrow_type, constfold::const_fold, culdesac::culdesac}, reorder::reorder, reuse::reuse, spill::spill, source::{ParsePosition, SourceSourceImpl, CombinedSourceSource, CombinedSourceSourceBuilder}, generate::generate, libcore::libcore::libcore_sources};

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

    pub(crate) fn parse_part(&mut self, position: &ParsePosition, path: &str, fixed: bool) -> Result<Vec<PTStatement>,String> {
        self.context += 1;
        let context = self.context;
        parse_earp(position,path,fixed,context)
    }

    fn parse_libcore(&mut self, position: &ParsePosition) -> Result<Vec<PTStatement>,String> {
        if self.flags.contains("no-libcore") { return Ok(vec![]); }
        self.context += 1;
        let context = self.context;
        Ok(vec![PTStatement { 
            value: PTStatementValue::Include("libcore".to_string(),true),
            position: position.clone(),
            context 
        }])
    }

    pub(crate) fn parse(&mut self, position: &ParsePosition, path: &str, fixed: bool) -> Result<Vec<PTStatement>,String> {
        let mut out = self.parse_libcore(position)?;
        out.append(&mut self.parse_part(position,path,fixed)?);
        Ok(out)
    }

    pub(crate) fn preprocess(&mut self, parse_tree: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
        preprocess(self,parse_tree)
    }

    pub(crate) fn build(&mut self, input: Vec<PTStatement>) -> Result<BuildTree,String> {
        PTStatement::to_build_tree(input)    
    }

    pub(crate) fn frontend(&mut self, filename: &str) -> Result<BuildTree,String> {
        let mut soso_builder = CombinedSourceSourceBuilder::new()?;
        soso_builder.add_fixed(libcore_sources());
        let soso = CombinedSourceSource::new(soso_builder);
        let position = ParsePosition::root(SourceSourceImpl::new(soso),"included");
        let stmts = self.parse(&position,filename,false)?;
        let stmts = self.preprocess(stmts)?;
        self.build(stmts)
    }

    pub(crate) fn middleend(&mut self, tree: &BuildTree) -> Result<Vec<Step>,String> {
        let bundles = build_unbundle(&tree)?;
        let (linear,next_register) = linearize(&tree,&bundles)?;
        let linear = reduce(&linear);
        let (broad,block_indexes) = broad_type(&tree,&linear)?;
        run_checking(&tree,&linear,&block_indexes)?;
        let mut narrow = narrow_type(&tree,&broad,&block_indexes,&linear)?;
        let opers = const_fold(&self,tree,&block_indexes,&linear);
        let opers = culdesac(tree,&block_indexes,&opers);
        let opers = reuse(tree,&block_indexes,&opers)?;
        let opers = reorder(&tree,&block_indexes,&opers)?;
        let opers = spill(next_register,&opers, &mut narrow);
        let opers = reorder(&tree,&block_indexes,&opers).expect("reorder failed");
        let steps = generate(&tree,&block_indexes,&narrow,&opers).expect("generate failed");
        Ok(steps)
    }

    pub fn compile(&mut self, filename: &str) -> Result<Vec<Step>,String> {
        let tree = self.frontend(filename)?;
        let step = self.middleend(&tree)?;
        Ok(step)
    }
}
