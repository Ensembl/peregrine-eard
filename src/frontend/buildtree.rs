use std::{sync::Arc, fmt};
use crate::{model::{ Variable, Check, Constant, ArgTypeSpec, OrBundle, TypedArgument, OrBundleRepeater}, codeblocks::{CodeDefinition, CodeBlock}};

#[derive(Debug,Clone)]
pub enum BTRegisterType {
    Normal,
    Bundle
}

#[derive(Clone)]
pub enum BTExpression {
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
pub enum BTLValue {
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
pub struct BTProcCall<R> {
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
pub struct BTFuncCall {
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
pub enum BTStatementValue {
    Define(usize),
    Declare(OrBundleRepeater<Variable>),
    Check(Variable,Check),
    BundledStatement(BTProcCall<OrBundleRepeater<BTLValue>>)
}

impl fmt::Debug for BTStatementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Define(v) => write!(f,"define {:?}",v),
            Self::Declare(v) => write!(f,"let {:?}",v),
            Self::Check(v, c) => write!(f,"{:?} <check> {:?}",v,c),
            Self::BundledStatement(v) => write!(f,"{:?}",v),
        }
    }
}

#[derive(Clone)]
pub struct BTStatement {
    pub value: BTStatementValue,
    pub file: Arc<Vec<String>>,
    pub line_no: usize
}

impl fmt::Debug for BTStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.file.as_ref().last().map(|x| x.as_str()).unwrap_or("");
        write!(f,"{}:{} {:?}",file,self.line_no,self.value)
    }
}

#[derive(Clone)]
pub struct BTFuncProcDefinition {
    pub(crate) position: (Arc<Vec<String>>,usize),
    pub(crate) args: Vec<OrBundle<TypedArgument>>,
    pub(crate) captures: Vec<OrBundle<Variable>>,
    pub(crate) block: Vec<BTStatement>,
    pub(crate) ret: Vec<OrBundle<BTExpression>>,
    pub(crate) ret_type: Option<Vec<ArgTypeSpec>>
}

impl BTFuncProcDefinition {
    pub(crate) fn at(&self) -> String {
        format!("{}:{}",self.position.0.last().map(|x| x.as_str()).unwrap_or("*anon*"),self.position.1)
    }
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

pub enum BTDefinitionVariety { Func, Proc }

#[derive(Clone)]
pub enum BTDefinition {
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

pub enum BTTopDefn<'a> {
    FuncProc(&'a BTFuncProcDefinition),
    Code(&'a CodeDefinition)
}

#[derive(Clone)]
pub struct BuildTree {
    pub(crate) statements: Vec<BTStatement>,
    pub(crate) definitions: Vec<BTDefinition>
}

impl BuildTree {
    pub(crate) fn new() -> BuildTree {
        BuildTree { statements: vec![], definitions: vec![] }
    }

    pub(super) fn add_definition(&mut self, defn: BTDefinition) -> usize {
        let id = self.definitions.len();
        self.definitions.push(defn);
        id
    }

    pub(crate) fn add_code(&mut self, id: usize, block: &CodeBlock) -> Result<(),String> {
        match &mut self.definitions[id] {
            BTDefinition::Code(c) => {
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
