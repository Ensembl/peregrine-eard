use std::{fmt, collections::HashMap};
use crate::{model::{CodeModifier, sepfmt, CodeArgument, FullConstant, CodeImplArgument, CodeReturn, Opcode}, middleend::broadtyping::BroadType};

#[derive(Clone)]
pub struct ImplBlock {
    pub arguments: Vec<CodeImplArgument>,
    pub results: Vec<CodeReturn>,
    pub command: Option<Opcode>,
}

impl ImplBlock {
    fn unifies(&self, args: &[Option<FullConstant>]) -> bool {
        for (defn,value) in self.arguments.iter().zip(args.iter()) {
            match (defn,value) {
                (CodeImplArgument::Register(_),_) => {},
                (CodeImplArgument::Constant(a), Some(FullConstant::Atomic(b))) => {
                    if a != b { return false; }
                },
                _ => { return false; }

            }
        }
        true
    }

    pub(crate) fn reg_reuse(&self) -> Result<Vec<(usize,usize)>,String> { // ret <- arg
        let mut reg_names = HashMap::new();
        for (i,arg) in self.arguments.iter().enumerate() {
            match arg {
                CodeImplArgument::Register(r) => {
                    reg_names.insert(r.reg_id,i);
                },
                CodeImplArgument::Constant(_) => {},
            }
        }
        let mut repeats = vec![];
        for (i,ret) in self.results.iter().enumerate() {
            match ret {
                CodeReturn::Register(_) => {},
                CodeReturn::Repeat(e) => {
                    let pos = reg_names.get(e).ok_or_else(|| 
                        "return repeat register not in argument list".to_string()
                    )?;
                    repeats.push((i,*pos));
                }
            }
        }
        Ok(repeats)
    }
}

impl fmt::Debug for ImplBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f," impl ({})",
            sepfmt(&mut self.arguments.iter(),", ","")
        )?;
        if self.results.len() > 0 {
            write!(f," -> ({}) ",
                sepfmt(&mut self.results.iter(),", ","")
            )?;
        }
        write!(f," {{\n{}\n}}\n\n",sepfmt(&mut self.command.iter(),"\n","  "))?;
        Ok(())
    }
}

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

    pub(crate) fn choose_imps(&self, args: &[Option<FullConstant>]) -> Vec<&ImplBlock> {
        self.impls.iter().filter(|imp| imp.unifies(args)).collect()
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
    ret_count: Option<usize>,
    blocks: Vec<CodeBlock>
}

impl CodeDefinition {
    pub(crate) fn new() -> CodeDefinition {
        CodeDefinition {
            ret_count: None,
            blocks: vec![]
        }
    }

    pub(crate) fn add(&mut self, alt: CodeBlock) -> Result<(),String> {
        let ret_count = self.ret_count.get_or_insert(alt.results.len());
        if *ret_count != alt.results.len() {
            return Err(format!("code blocks must match in return value count: {} vs {}",alt.results.len(),ret_count));
        }
        self.blocks.push(alt);
        Ok(())
    }

    pub(crate) fn ret_count(&self) -> usize {
        self.ret_count.expect("query on empty code block")
    }

    pub(crate) fn broad_typing(&self, src: &[BroadType]) -> Result<(Vec<BroadType>,usize),String> {
        for (i,block) in self.blocks.iter().enumerate() {
            if let Some(dst) = block.broad_typing(src)? {
                return Ok((dst,i));
            }
        }
        Err(format!("could not find appropriate implementation"))
    }

    pub(crate) fn get_block(&self, index: usize) -> &CodeBlock { &self.blocks[index] }
}

impl fmt::Debug for CodeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for block in &self.blocks {
            write!(f,"{:?}\n",block)?;
        }
        Ok(())
    }
}
