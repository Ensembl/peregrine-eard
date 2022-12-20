use std::{collections::{HashMap}};
use crate::{frontend::{parsetree::{PTStatement, PTExpression}}, model::{FullConstant, ParsePosition}, libcore::libcore::libcore_add};
use crate::model::{OrBundleRepeater};

pub struct EarpCompiler {
    source_loader: Box<dyn Fn(&str) -> Result<String,String>>,
    block_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],&ParsePosition,usize) -> Result<Vec<PTStatement>,String>>>,
    expression_macros: HashMap<String,Box<dyn Fn(&[OrBundleRepeater<PTExpression>],usize) -> Result<PTExpression,String>>>,
    constant_folder: HashMap<String,Box<dyn Fn(&[Option<FullConstant>]) -> Option<Vec<FullConstant>>>>
}

impl EarpCompiler {
    pub fn new() -> Result<EarpCompiler,String> {
        let mut out = EarpCompiler {
            source_loader: Box::new(|_| { Err("No source loader set".to_string()) }),
            block_macros: HashMap::new(),
            expression_macros: HashMap::new(),
            constant_folder: HashMap::new()
        };
        libcore_add(&mut out)?;
        Ok(out)
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
