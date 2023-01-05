use crate::{ frontend::{parsetree::{PTStatement, PTStatementValue}, buildtree::BuildTree, preprocess::preprocess, parser::{parse_eard}}, unbundle::{buildunbundle::build_unbundle, linearize::linearize}, middleend::{reduce::reduce, checking::run_checking, broadtyping::broad_type, narrowtyping::narrow_type, constfold::const_fold, culdesac::culdesac, reuse::reuse, spill::spill, reorder::reorder, generate::generate, large::large}, libcore::libcore::libcore_sources, model::{step::Step, compiled::{Metadata, CompiledCode}}};
use super::{compiler::EardCompiler, source::{CombinedSourceSourceBuilder, FixedSourceSource, ParsePosition, CombinedSourceSource, SourceSourceImpl}, compiled::make_program};

pub struct EardCompilation<'a> {
    pub(crate) compiler: &'a EardCompiler,
    soso_builder: CombinedSourceSourceBuilder,
    context: usize}

impl<'a> EardCompilation<'a> {
    pub fn new(compiler: &'a EardCompiler) -> Result<EardCompilation,String> {
        let mut soso_builder = CombinedSourceSourceBuilder::new()?;
        soso_builder.add_fixed(&libcore_sources());
        for source in compiler.sources() {
            soso_builder.add_fixed(source);
        }
        Ok(EardCompilation {
            compiler,
            soso_builder,
            context: 0,
        })
    }

    pub(crate) fn compiler(&self) -> &EardCompiler { &self.compiler }

    pub(crate) fn parse_part(&mut self, position: &ParsePosition, path: &str, fixed: bool) -> Result<Vec<PTStatement>,String> {
        self.context += 1;
        let context = self.context;
        parse_eard(position,path,fixed,self.compiler().optimise(),context)
    }

    fn add_libcore(&mut self, position: &ParsePosition) -> Result<Vec<PTStatement>,String> {
        if self.compiler().has_flag("no-libcore") { return Ok(vec![]); }
        self.context += 1;
        let context = self.context;
        Ok(vec![PTStatement { 
            value: PTStatementValue::Include("libcore".to_string(),true),
            position: position.clone(),
            context 
        }])
    }

    pub(crate) fn parse(&mut self, position: &ParsePosition, path: &str, fixed: bool) -> Result<Vec<PTStatement>,String> {
        let mut out = self.add_libcore(position)?;
        out.append(&mut self.parse_part(position,path,fixed)?);
        Ok(out)
    }

    pub(crate) fn preprocess(&mut self, parse_tree: Vec<PTStatement>) -> Result<Vec<PTStatement>,String> {
        preprocess(self,parse_tree)
    }

    pub(crate) fn build(&mut self, input: Vec<PTStatement>) -> Result<BuildTree,String> {
        PTStatement::to_build_tree(input,self.compiler().target_version())
    }

    pub(crate) fn frontend(&mut self, filename: &str) -> Result<BuildTree,String> {
        let soso = CombinedSourceSource::new(&self.soso_builder);
        let position = ParsePosition::root(SourceSourceImpl::new(soso),"included");
        let stmts = self.parse(&position,filename,false)?;
        let stmts = self.preprocess(stmts)?;
        self.build(stmts)
    }

    pub(crate) fn middleend(&mut self, tree: &BuildTree) -> Result<(Vec<Step>,Metadata),String> {
        let verbose = self.compiler.verbose();
        let bundles = build_unbundle(&tree)?;
        let (linear,mut allocator,metadata) = linearize(&tree,&bundles,verbose)?;
        let linear = reduce(&linear,verbose)?;
        let (mut broad,block_indexes) = broad_type(&tree,&linear)?;
        let linear = run_checking(&tree,&linear,&block_indexes,&mut allocator,&mut broad,verbose)?;
        let mut narrow = narrow_type(&tree,&block_indexes,&broad,&linear)?;
        let opers = const_fold(&self,tree,&block_indexes,&narrow,&linear,verbose);
        let opers = culdesac(tree,&block_indexes,&opers,verbose);
        let opers = reuse(tree,&broad,&narrow,&block_indexes,&opers,verbose)?;
        let opers = reorder(&tree,&block_indexes,&opers)?;
        let opers = spill(&mut allocator,&opers, &mut narrow);
        let mut opers = reorder(&tree,&block_indexes,&opers).expect("reorder failed");
        if !self.compiler().has_flag("no-call-up") {
            opers = large(tree,&block_indexes,&mut allocator,&opers)?;
        }
        let steps = generate(&tree,&block_indexes,&narrow,&opers,verbose).expect("generate failed");
        Ok((steps,metadata))
    }

    pub fn compile(&mut self, filename: &str) -> Result<CompiledCode,String> {
        let tree = self.frontend(filename)?;
        let (steps,metadata) = self.middleend(&tree)?;
        let program = make_program(&steps,&metadata);
        Ok(program)
    }
}
