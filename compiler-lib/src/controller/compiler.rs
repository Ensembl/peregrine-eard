use std::{collections::{HashMap, HashSet}};
use crate::{frontend::{parsetree::{PTStatement, PTExpression}, femodel::OrBundleRepeater}, libcore::libcore::libcore_add, model::constants::FullConstant};

use super::source::{ParsePosition, FixedSourceSource};

pub struct EardCompiler {
    sources: Vec<FixedSourceSource>,
    flags: HashSet<String>,
    optimise: bool,
    verbose: bool,
    target_version: Option<u32>,
    block_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],&ParsePosition,usize) -> Result<Vec<PTStatement>,String>>>,
    expression_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],usize) -> Result<PTExpression,String>>>,
    constant_folder: HashMap<String,Box<dyn Fn(&[Option<FullConstant>]) -> Option<Vec<FullConstant>>>>
}

impl EardCompiler {
    pub fn new() -> Result<EardCompiler,String> {
        let mut out = EardCompiler {
            sources: vec![],
            flags: HashSet::new(),
            optimise: false,
            verbose: false,
            target_version: None,
            block_macros: HashMap::new(),
            expression_macros: HashMap::new(),
            constant_folder: HashMap::new()
        };
        libcore_add(&mut out)?;
        Ok(out)
    }

    pub fn add_source(&mut self, source: FixedSourceSource) {
        self.sources.push(source);
    }

    pub fn set_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string());
    }

    pub fn set_verbose(&mut self, yn: bool) { self.verbose = yn; }
    pub fn verbose(&self) -> bool { self.verbose }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    pub fn set_target_version(&mut self, version: u32) { self.target_version = Some(version); }
    pub fn target_version(&self) -> Option<u32> { self.target_version }

    pub fn set_optimise(&mut self, yn: bool) { self.optimise = yn; }
    pub fn optimise(&self) -> bool { self.optimise }

    fn check_macro_name_unused(&self, name: &str) -> Result<(),String> {
        if self.block_macros.contains_key(name) || self.expression_macros.contains_key(name) {
            Err(format!("Duplicate macro definition '{}'",name))
        } else {
            Ok(())
        }
    }

    pub fn add_block_macro<F>(&mut self, name: &str, macro_cb: F) -> Result<(),String>
            where F: Fn(&[OrBundleRepeater<PTExpression>],&ParsePosition,usize) -> Result<Vec<PTStatement>,String> + 'static {
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

    pub fn add_constant_folder<F>(&mut self, name: &str, cb: F) -> Result<(),String>
            where F: Fn(&[Option<FullConstant>]) -> Option<Vec<FullConstant>> + 'static {
        if self.constant_folder.contains_key(name) {
            return Err(format!("Duplicate constant folder '{}'",name));
        }
        self.constant_folder.insert(name.to_string(),Box::new(cb));
        Ok(())
    }

    pub(crate) fn sources(&self) -> &[FixedSourceSource] { &self.sources }

    pub(crate) fn fold(&self, name: &str, input: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
        self.constant_folder.get(name).and_then(|cb| {
            (cb)(input)
        })
    }

    pub(crate) fn apply_block_macro(&self, name: &str, args: &[OrBundleRepeater<PTExpression>], pos: &ParsePosition, context: usize) -> Result<Vec<PTStatement>,String> {
        let cb = self.block_macros.get(name).ok_or_else(|| format!("No such block macro \"{:?}\"",name))?;
        cb(args,pos,context)
    }

    pub(crate) fn apply_expression_macro(&self, name: &str, args: &[OrBundleRepeater<PTExpression>], context: usize) -> Result<PTExpression,String> {
        let cb = self.expression_macros.get(name).ok_or_else(|| format!("No such expression macro \"{:?}\"",name))?;
        cb(args,context)
    }
}
