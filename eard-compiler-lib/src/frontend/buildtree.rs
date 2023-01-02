use std::{fmt::{self, Display}, collections::HashMap};
use crate::{model::{constants::Constant, checkstypes::{Check, ArgTypeSpec, TypedArgument}, codeblocks::{CodeDefinition, CodeBlock, CodeModifier}}, controller::source::ParsePosition};

use super::femodel::{OrBundleRepeater, OrBundle};

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Variable {
    pub prefix: Option<String>,
    pub name: String
}

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f,"{}.{}",prefix,self.name)
        } else {
            write!(f,"{}",self.name)
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}",self)
    }
}

#[derive(Debug,Clone)]
pub(crate) enum BTRegisterType {
    Normal,
    Bundle
}

#[derive(Clone)]
pub(crate) enum BTExpression {
    Constant(Constant),
    Variable(Variable),
    RegisterValue(usize,BTRegisterType),
    Function(BTFuncCall)
}

impl fmt::Debug for BTExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(v) => write!(f,"{:?}",v),
            Self::Variable(v) => write!(f,"{:?}",v),
            Self::RegisterValue(v,BTRegisterType::Normal) => write!(f,"r{}",v),
            Self::RegisterValue(v,BTRegisterType::Bundle) => write!(f,"*r{}",v),
            Self::Function(v) => write!(f,"{:?}",v)
        }
    }
}

// XXX block macros
#[derive(Clone)]
pub(crate) enum BTLValue {
    Variable(Variable),
    Register(usize,BTRegisterType)
}

impl fmt::Debug for BTLValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(v) => write!(f,"{:?}",v),
            Self::Register(v,BTRegisterType::Normal) => write!(f,"r{}",v),
            Self::Register(v,BTRegisterType::Bundle) => write!(f,"*r{}",v),
        }
    }
}


#[derive(Clone)]
pub(crate) struct BTProcCall<R> {
    pub(crate) proc_index: Option<usize>,
    pub(crate) args: Vec<OrBundleRepeater<BTExpression>>,
    pub(crate) rets: Option<Vec<R>>,
    pub(crate) call_index: usize
}

impl<R: fmt::Debug> fmt::Debug for BTProcCall<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rets) = &self.rets {
            write!(f,"(")?;
            for (i,ret) in rets.iter().enumerate() {
                write!(f,"{}{:?}",if i>0 { " " } else { "" },ret)?;
            }
            write!(f,") <- ")?;
        }
        if let Some(proc) = self.proc_index {
            write!(f,"({}#{}",proc,self.call_index)?;    
        } else {
            write!(f,"#{}",self.call_index)?;
        }
        for arg in &self.args {
            write!(f," {:?}",arg)?;
        }
        if self.proc_index.is_some() {
            write!(f,")")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct BTFuncCall {
    pub(crate) func_index: usize,
    pub(crate) args: Vec<OrBundleRepeater<BTExpression>>,
    pub(crate) call_index: usize
}

impl fmt::Debug for BTFuncCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"({}#{}",self.func_index,self.call_index)?;
        for arg in &self.args {
            write!(f," {:?}",arg)?;
        }
        write!(f,")")
    }
}

#[derive(Clone)]
pub(crate) enum BTStatementValue {
    Header(String,String,u32),
    Version(String,u32,u32),
    Define(usize),
    Declare(OrBundleRepeater<Variable>),
    Check(Variable,Check),
    BundledStatement(BTProcCall<OrBundleRepeater<BTLValue>>),
    Entry(usize,String)
}

impl fmt::Debug for BTStatementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header(group,name,version) => write!(f,"program {:?} {:?} {:?}",group,name,version),
            Self::Version(name,a,b) => write!(f,"version {:?} {} {}",name,a,b),
            Self::Define(v) => write!(f,"define {:?}",v),
            Self::Entry(v,s) => write!(f,"entry {:?} {:?}",v,s),
            Self::Declare(v) => write!(f,"let {:?}",v),
            Self::Check(v, c) => write!(f,"{:?} <check> {:?}",v,c),
            Self::BundledStatement(v) => write!(f,"{:?}",v),
        }
    }
}

#[derive(Clone)]
pub(crate) struct BTStatement {
    pub(crate) value: BTStatementValue,
    pub(crate) position: ParsePosition
}

impl fmt::Debug for BTStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {:?}",self.position.last_str(),self.value)
    }
}

#[derive(Clone)]
pub(crate) struct BTFuncProcDefinition {
    pub(crate) position: ParsePosition,
    pub(crate) args: Vec<OrBundle<TypedArgument>>,
    pub(crate) captures: Vec<OrBundle<Variable>>,
    pub(crate) block: Vec<BTStatement>,
    pub(crate) ret: Vec<OrBundle<BTExpression>>,
    pub(crate) ret_type: Option<Vec<ArgTypeSpec>>,
    pub(crate) entry: bool
}

impl fmt::Debug for BTFuncProcDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"(")?;
        for (i,arg) in self.args.iter().enumerate() {
            write!(f,"{}{:?}",if i>0 { "," } else { "" },arg)?;
        }
        write!(f,") ")?;
        if let Some(ret_type) = &self.ret_type {
            write!(f,"-> (")?;
            for (i,ret_type) in ret_type.iter().enumerate() {
                write!(f,"{}{:?}",if i>0 { "," } else { "" },ret_type)?;
            }
            write!(f,") ")?;
        }
        write!(f,"{{\n")?;
        for capture in &self.captures {
            write!(f,"  capture {:?}\n",capture)?;
        }
        for stmt in &self.block {
            write!(f,"  {:?}\n",stmt)?;
        }
        write!(f,"  (")?;
        for (i,ret) in self.ret.iter().enumerate() {
            write!(f,"{}{:?}",if i>0 {","} else {""},ret)?;
        }
        write!(f,")\n}}\n")?;
        Ok(())
    }
}

pub(crate) enum BTDefinitionVariety { Func, Proc }

#[derive(Clone)]
pub(crate) enum BTDefinition {
    Func(BTFuncProcDefinition),
    Proc(BTFuncProcDefinition),
    Code(CodeDefinition)
}

impl fmt::Debug for BTDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
            Self::Func(v) => write!(f,"func {:?}",v),
            Self::Proc(v) => write!(f,"proc {:?}",v),
            Self::Code(v) => write!(f,"{:?}",v)
        }
    }
}

pub(crate) enum BTTopDefn<'a> {
    FuncProc(&'a BTFuncProcDefinition),
    Code(&'a CodeDefinition)
}

#[derive(Clone)]
pub(crate) struct BuildTree {
    pub(crate) statements: Vec<BTStatement>,
    definitions: Vec<BTDefinition>,
    specials: HashMap<String,usize>,
    entries: Vec<(usize,String)>
}

impl BuildTree {
    pub(crate) fn new() -> BuildTree {
        BuildTree { statements: vec![], definitions: vec![], specials: HashMap::new(), entries: vec![] }
    }

    pub(crate) fn finish(&mut self) {
        for (idx,name) in &self.entries {
            self.statements.push(BTStatement {
                position: ParsePosition::empty("included"),
                value: BTStatementValue::Entry(*idx,name.to_string()) 
            });
        }
    }

    pub(crate) fn get_special(&self, name: &str) -> usize {
        *self.specials.get(name).expect(&format!("missing special '{}'",name))
    }

    pub(super) fn set_entry(&mut self, id: usize, name: &str) {
        self.entries.push((id,name.to_string()));
    }

    pub(super) fn add_definition(&mut self, defn: BTDefinition) -> usize {
        let id = self.definitions.len();
        self.definitions.push(defn);
        id
    }

    fn extract_special(&self, mods: &[CodeModifier]) -> Option<String> {
        mods.iter().filter_map(|m| {
            match m {
                CodeModifier::Special(s) => Some(s.to_string()),
                _ => None
            }
        }).next()
    }

    pub(crate) fn add_code(&mut self, id: usize, block: &CodeBlock) -> Result<(),String> {
        let special = self.extract_special(&block.modifiers);
        match &mut self.definitions[id] {
            BTDefinition::Code(c) => {
                if let Some(special) = special {
                    self.specials.insert(special,id);
                }
                c.add(block.clone())?;
            },
            _ => {
                panic!("Incorrectly indexed definition");
            }
        }
        Ok(())
    }

    pub(super) fn add_statement(&mut self, stmt: BTStatement) {
        self.statements.push(stmt);
    }

    pub(crate) fn get_function(&self, f: &BTFuncCall) -> Result<&BTFuncProcDefinition,String> {
        Ok(match &self.definitions[f.func_index] {
            BTDefinition::Func(f) => f,
            _ => { return Err(format!("expected function, got non-function")); }
        })
    }

    pub(crate) fn get_by_index<'a>(&'a self, index: usize) -> Result<BTTopDefn<'a>,String> {
        Ok(match &self.definitions[index] {
            BTDefinition::Proc(p) => BTTopDefn::FuncProc(p),
            BTDefinition::Func(f) => BTTopDefn::FuncProc(f),
            BTDefinition::Code(c) => BTTopDefn::Code(c)
        })
    }

    pub(crate) fn get_any<'a>(&'a self, p: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Option<BTTopDefn<'a>>,String> {
        p.proc_index.map(|index| self.get_by_index(index)).transpose()
    }
}

impl fmt::Debug for BuildTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i,defn) in self.definitions.iter().enumerate() {
            write!(f,"{}:{:?}\n",i,defn)?;
        }
        for stmt in &self.statements {
            write!(f,"{:?}\n",stmt)?;
        }
        Ok(())
    }
}
