use std::{fmt, collections::HashMap};
use crate::{model::{checkstypes::{Check, TypeSpec}, constants::{FullConstant, Constant}, compiled::Opcode}, middleend::{narrowtyping::NarrowType, broadtyping::BroadType}, test::testutil::sepfmt};

#[derive(Clone,PartialEq,Eq)]
pub enum CodeModifier {
    World,
    Fold(String),
    Special(String)
}

impl fmt::Debug for CodeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeModifier::World => write!(f,"world")?,
            CodeModifier::Fold(s) => write!(f,"fold({})",s)?,
            CodeModifier::Special(s) => write!(f,"special({})",s)?
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeArgument {
    pub arg_type: TypeSpec,
    pub checks: Vec<Check>
}

impl fmt::Debug for CodeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?} {}",
            self.arg_type,
            sepfmt(&mut self.checks.iter()," ","")
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeImplVariable {
    pub reg_id: usize,
    pub arg_type: TypeSpec
}

impl fmt::Debug for CodeImplVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"r{} : {:?}",
            self.reg_id,
            self.arg_type
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum CodeImplArgument {
    Register(CodeImplVariable),
    Constant(Constant)
}

impl fmt::Debug for CodeImplArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(r) => write!(f,"{:?}",r),
            Self::Constant(c) => write!(f,"{:?}",c),
        }
    }
}

#[derive(Clone)]
pub enum CodeReturn {
    Register(CodeImplVariable),
    Repeat(usize)
}

impl fmt::Debug for CodeReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(r) => write!(f,"{:?}",r),
            Self::Repeat(r) => write!(f,"r{}",*r),
        }
    }
}

#[derive(Clone)]
pub struct ImplBlock {
    pub arguments: Vec<CodeImplArgument>,
    pub results: Vec<CodeReturn>,
    pub command: Option<Opcode>,
}

impl ImplBlock {
    fn unifies_constant(&self, args: &[Option<FullConstant>]) -> bool {
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

    fn unifies_types(&self, narrow: &[NarrowType]) -> bool {
        let mut wilds = HashMap::new();
        for (defn,narrow) in self.arguments.iter().zip(narrow.iter()) {
            let want_narrow = match defn {
                CodeImplArgument::Register(r) => {
                    r.arg_type.clone()
                },
                CodeImplArgument::Constant(c) => {
                    TypeSpec::Atomic(c.to_atomic_type())
                }
            };
            match &want_narrow {
                TypeSpec::Atomic(a) => {
                    let atomic = NarrowType::Atomic(a.clone());
                    if &atomic != narrow { return false; }
                },
                TypeSpec::Sequence(s) => {
                    let seq = NarrowType::Sequence(s.clone());
                    if &seq != narrow { return false; }
                },
                TypeSpec::Wildcard(w) => {
                    if let Some(value) = wilds.get(w) {
                        if value != narrow { return false; }
                    } else {
                        wilds.insert(w.to_string(),narrow.clone());
                    }
                },
                TypeSpec::SequenceWildcard(w) => {
                    if let Some(value) = wilds.get(w) {
                        let value = match value {
                            NarrowType::Atomic(a) => NarrowType::Sequence(a.clone()),
                            NarrowType::Sequence(_) => { return false; },
                        };
                        if &value != narrow { return false; }
                    } else {
                        match narrow {
                            NarrowType::Atomic(_) => { return false; },
                            NarrowType::Sequence(a) => {
                                wilds.insert(w.to_string(),NarrowType::Atomic(a.clone()));
                            },
                        };
                    }
                },
            }
        }
        true
    }

    fn unifies_modify(&self, modify: &[Option<usize>]) -> bool {
        let mut reg_to_class = HashMap::new();
        for (defn,value) in self.arguments.iter().zip(modify.iter()) {
            match defn {
                CodeImplArgument::Register(r) => {
                    reg_to_class.insert(r.reg_id,value);
                },
                CodeImplArgument::Constant(_) => {}
            }
        }
        let mut reg_classes = HashMap::new();
        for ret in self.results.iter() {
            match ret {
                CodeReturn::Register(_) => {},
                CodeReturn::Repeat(reg) => {
                    let reg_class = *reg_to_class.get(reg).expect("missing register in opcode match");
                    if let Some(reg_class) = reg_class {
                        /* Must be in same reg class as other uses */
                        if let Some(old_reg_class) = reg_classes.get(reg).cloned() {
                            if old_reg_class != reg_class { return false; }
                        } else {
                            reg_classes.insert(*reg,reg_class);
                        }
                    } else {
                        /* no register modify class so we can't accept a repeat here */
                        return false;
                    }
                },
            }
        }
        true
    }

    fn unifies(&self, args: &[Option<FullConstant>], narrow: Option<&[NarrowType]>, modify: Option<&Vec<Option<usize>>>) -> bool {
        if !self.unifies_constant(args) { return false; }
        if let Some(narrow) = narrow {
            if !self.unifies_types(narrow) { return false; }
        }
        if let Some(regs) = modify {
            if !self.unifies_modify(regs) { return false; }
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

    pub(crate) fn add_impls(&mut self, impls: Vec<ImplBlock>) -> Result<(),String> {
        for imp in &impls {
            if imp.arguments.len() != self.arguments.len() {
                return Err(format!("mismatch in arg list between code block and impl"));
            }
            if imp.results.len() != self.results.len() {
                return Err(format!("mismatch in return list between code block and impl"));
            }
        }
        self.impls = impls;
        Ok(())
    }

    pub(crate) fn choose_imps(&self, args: &[Option<FullConstant>], narrow: Option<&[NarrowType]>, modify: Option<&Vec<Option<usize>>>) -> Vec<&ImplBlock> {
        self.impls.iter().filter(|imp| imp.unifies(args,narrow,modify)).collect()
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
pub(crate) struct CodeDefinition {
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
