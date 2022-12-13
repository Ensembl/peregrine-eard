use std::{fmt, collections::HashMap};
use crate::{model::{CodeModifier, sepfmt, ImplBlock, CodeArgument, TypeSpec}, typing::BroadType};

#[derive(Clone)]
pub struct CodeBlock {
    pub name: String,
    pub arguments: Vec<CodeArgument>,
    pub results: Vec<CodeArgument>,
    pub impls: Vec<ImplBlock>,
    pub modifiers: Vec<CodeModifier>
}

impl CodeBlock {
    fn broad_typing(&self, src: &[BroadType]) -> Result<Option<Vec<BroadType>>,String> {
        let mut wilds = HashMap::new();
        /* go through arguments to check for applicatility and set any wilds */
        if src.len() != self.arguments.len() { return Ok(None); }
        for (src,spec) in src.iter().zip(self.arguments.iter()) {
            let broad_spec = BroadType::from_typespec(&spec.arg_type);
            let wanted = broad_spec.as_ref().map_or_else(|w| {
                &*wilds.entry(w.to_string()).or_insert_with(|| src.clone())
            },|w| w);            
            if src != wanted {
                return Ok(None);
            }
        }
        /* go through results setting types */
        let typing = self.results.iter().map(|s| {
            BroadType::from_typespec(&s.arg_type).map_err(|w| {
                wilds.get(&w).cloned().ok_or_else(|| format!("unbound wildcard {}",w))
            }).map_or_else(|x| x,|x| Ok(x))
        }).collect::<Result<Vec<_>,_>>()?;
        Ok(Some(typing))
    }
}

impl fmt::Debug for CodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",sepfmt(&mut self.modifiers.iter()," ",""))?;
        write!(f," code {}({})",
            self.name,
            sepfmt(&mut self.arguments.iter(),", ","")
        )?;
        if self.results.len() > 0 {
            write!(f," -> ({}) ",
                sepfmt(&mut self.results.iter(),", ","")
            )?;
        }
        write!(f," {{\n{}\n}}\n\n",sepfmt(&mut self.impls.iter(),"\n","  "))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeDefinition {
    world: bool,
    ret_count: Option<usize>,
    blocks: Vec<CodeBlock>
}

impl CodeDefinition {
    pub(crate) fn new() -> CodeDefinition {
        CodeDefinition {
            world: false,
            ret_count: None,
            blocks: vec![]
        }
    }

    pub(crate) fn add(&mut self, alt: CodeBlock) -> Result<(),String> {
        let ret_count = self.ret_count.get_or_insert(alt.results.len());
        if *ret_count != alt.results.len() {
            return Err(format!("code blocks must match in return value count: {} vs {}",alt.results.len(),ret_count));
        }
        if alt.modifiers.contains(&CodeModifier::World) {
            self.world = true;
        }
        self.blocks.push(alt);
        Ok(())
    }

    pub(crate) fn ret_count(&self) -> usize {
        self.ret_count.expect("query on empty code block")
    }

    pub(crate) fn world(&self) -> bool { self.world }

    pub(crate) fn broad_typing(&self, src: &[BroadType]) -> Result<Vec<BroadType>,String> {
        for block in &self.blocks {
            if let Some(dst) = block.broad_typing(src)? {
                return Ok(dst);
            }
        }
        Err(format!("could not find appropriate implementation"))
    }
}

impl fmt::Debug for CodeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for block in &self.blocks {
            write!(f,"{:?}\n",block)?;
        }
        Ok(())
    }
}
