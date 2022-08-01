use std::{sync::Arc, collections::{BTreeMap}};

use crate::model::{ Variable, Check, CallArg, Constant, ArgTypeSpec, OrBundle, TypedArgument, CodeBlock};

#[derive(Debug,Clone)]
pub enum BTExpression {
    Constant(Constant),
    Variable(Variable),
    RegisterValue(usize),
    Function(BTFuncCall)
}

// XXX block macros
#[derive(Debug,Clone)]
pub enum BTLValue {
    Variable(Variable),
    Register(usize),
    Repeater(String)
}

#[derive(Debug,Clone)]
pub struct BTProcCall {
    proc_index: Option<usize>,
    args: Vec<CallArg<BTExpression>>,
    rets: Option<Vec<BTLValue>>
}

#[derive(Debug,Clone)]
pub struct BTFuncCall {
    pub(crate) func_index: usize,
    pub(crate) args: Vec<CallArg<BTExpression>>
}

#[derive(Debug,Clone)]
pub enum BTDeclare {
    Variable(Variable),
    Repeater(String),
    Bundle(String)
}

#[derive(Debug,Clone)]
pub enum BTStatementValue {
    Define(usize),
    Declare(BTDeclare),
    Check(Variable,Check),
    Capture(OrBundle<Variable>),
    Statement(BTProcCall)
}

#[derive(Debug,Clone)]
pub struct BTStatement {
    pub value: BTStatementValue,
    pub file: Arc<Vec<String>>,
    pub line_no: usize
}

#[derive(Debug,Clone)]
pub struct BTCodeDefinition {
    blocks: Vec<CodeBlock>
}

impl BTCodeDefinition {
    fn add(&mut self, alt: CodeBlock) {
        self.blocks.push(alt);
    }
}

struct CurrentFuncProcDefinition {
    block: Vec<BTStatement>,
    name: String,
    export: bool,
    args: Vec<OrBundle<TypedArgument>>,
    variety: BTDefinitionVariety,
    ret_type: Option<Vec<ArgTypeSpec>>, // exactly one for functions
}

impl CurrentFuncProcDefinition {
    fn to_funcproc(&self, ret: &[OrBundle<BTExpression>]) -> BTFuncProcDefinition {
        BTFuncProcDefinition {
            args: self.args.clone(),
            ret_type: self.ret_type.clone(),
            ret: ret.to_vec(),
            block: self.block.clone()
        }
    }
}

#[derive(Debug,Clone)]
enum DefName {
    Func(usize),
    Proc(usize),
    Code(usize)
}

pub struct BuildContext {
    location: (Arc<Vec<String>>,usize),
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),DefName>,
    next_register: usize,
    funcproc_target: Option<CurrentFuncProcDefinition>
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext {
            location: (Arc::new(vec!["*anon*".to_string()]),0),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0,
            funcproc_target: None
        }
    }

    pub fn push_funcproc_target(&mut self, is_proc: bool, name: &str,
            args: &[OrBundle<TypedArgument>],
            ret_type: Option<Vec<ArgTypeSpec>>,
            export: bool, bt: &mut BuildTree) {
        let variety = if is_proc { BTDefinitionVariety::Proc } else { BTDefinitionVariety::Func };
        self.funcproc_target = Some(CurrentFuncProcDefinition {
            name: name.to_string(),
            args: args.to_vec(),
            export, variety,
            block: vec![],
            ret_type
        });
    }

    pub fn pop_funcproc_target(&mut self, ret: &[OrBundle<BTExpression>], bt: &mut BuildTree) -> Result<(),String> {
        let ctx = self.funcproc_target.take().expect("pop without push");
        let defn = match &ctx.variety {
            BTDefinitionVariety::Func => {                
                bt.define_funcproc(&ctx.name,ctx.to_funcproc(ret),false,self,ctx.export)?
            },
            BTDefinitionVariety::Proc => {
                bt.define_funcproc(&ctx.name,ctx.to_funcproc(ret),true,self,ctx.export)?
            }
        };
        self.add_statement(bt,defn);
        Ok(())
    }

    pub fn set_file_context(&mut self, context: usize) {
        self.file_context = context;
    }

    pub fn set_location(&mut self, file: &Arc<Vec<String>>, line_no: usize) {
        self.location = (file.clone(),line_no);
    }

    pub fn location(&self) -> (&[String],usize) {
        (self.location.0.as_ref(),self.location.1)
    }

    fn lookup(&self, name: &str) -> Result<DefName,String> {
        self.defnames
            .get(&(Some(self.file_context),name.to_string()))
            .or_else(||
                self.defnames.get(&(None,name.to_string()))
            )
            .ok_or_else(|| format!("No such function/procedure {}",name))
            .cloned()
    }

    pub(crate) fn lookup_func(&self, name: &str) -> Result<usize,String> {
        match self.lookup(name) {
            Ok(DefName::Func(x)) => Ok(x),
            _ => Err(format!("cannot find function {}",name))
        }
    }

    pub(crate) fn lookup_proc(&self, name: &str) -> Result<usize,String> {
        match self.lookup(name) {
            Ok(DefName::Proc(x)) => Ok(x),
            _ => Err(format!("cannot find procedure {}",name))
        }
    }

    pub(crate) fn allocate_register(&mut self) -> usize {
        self.next_register += 1;
        self.next_register
    }

    pub(crate) fn add_statement(&mut self, bt: &mut BuildTree, value: BTStatementValue) -> Result<(),String> {
        let stmt = BTStatement {
            value,
            file: self.location.0.clone(),
            line_no: self.location.1
        };
        if let Some(target) = self.funcproc_target.as_mut() {
            target.block.push(stmt);
        } else {
            bt.statements.push(stmt);
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct BTFuncProcDefinition {
    args: Vec<OrBundle<TypedArgument>>,
    block: Vec<BTStatement>,
    ret: Vec<OrBundle<BTExpression>>,
    ret_type: Option<Vec<ArgTypeSpec>>
}

pub enum BTDefinitionVariety { Func, Proc }

#[derive(Debug,Clone)]
pub enum BTDefinition {
    Func(BTFuncProcDefinition),
    Proc(BTFuncProcDefinition),
    Code(BTCodeDefinition)
}

#[derive(Debug,Clone)]
pub struct BuildTree {
    statements: Vec<BTStatement>,
    definitions: Vec<BTDefinition>
}

impl BuildTree {
    pub(crate) fn new() -> BuildTree {
        BuildTree { statements: vec![], definitions: vec![] }
    }

    fn define_funcproc(&mut self, name: &str, definition: BTFuncProcDefinition, is_proc: bool, bc: &mut BuildContext, export: bool) -> Result<BTStatementValue,String> {
        let id = self.definitions.len();
        let name_id = if is_proc { DefName::Proc(id) } else { DefName::Func(id) };
        let context = if export { None } else { Some(bc.file_context) };
        let key = (context,name.to_string());
        if bc.defnames.contains_key(&key) {
            return Err(format!("duplciate definition for {}",name));
        }
        bc.defnames.insert(key,name_id);
        let definition = if is_proc { BTDefinition::Proc(definition) } else { BTDefinition::Func(definition) };
        self.definitions.push(definition);
        Ok(BTStatementValue::Define(id))
    }

    pub(crate) fn define_code(&mut self, block: &CodeBlock, bc: &mut BuildContext) -> Result<(),String> {
        let key = (Some(bc.file_context),block.name.to_string());
        if !bc.defnames.contains_key(&key) {
            let id = self.definitions.len();
            let name_id = DefName::Code(id);
            bc.defnames.insert(key.clone(),name_id);
            self.definitions.push(BTDefinition::Code(BTCodeDefinition {
                blocks: vec![]
            }));
        }
        match bc.defnames.get(&key) {
            Some(DefName::Code(id)) => {
                match &mut self.definitions[*id] {
                    BTDefinition::Code(c) => {
                        c.add(block.clone());
                    },
                    _ => {
                        panic!("Incorrectly indexed definition");
                    }
                }
                Ok(())
            },
            _ => {
                Err(format!("duplciate definition for {}",block.name))
            }
        }
    }

    pub(crate) fn declare(&mut self, declaration: BTDeclare, bc: &BuildContext) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Declare(declaration))
    }

    pub(crate) fn check(&mut self, variable: &Variable, check: &Check, bc: &BuildContext) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Check(variable.clone(),check.clone()))
    }

    pub(crate) fn statement(&mut self, defn: Option<usize>, args: Vec<CallArg<BTExpression>>, rets: Option<Vec<BTLValue>>, bc: &BuildContext) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Statement(BTProcCall {
            proc_index: defn,
            args, rets
        }))
    }

    pub(crate) fn function_call(&self, defn: usize, args: Vec<CallArg<BTExpression>>, bc: &BuildContext) -> Result<BTFuncCall,String> {
        Ok(match &self.definitions[defn] {
            BTDefinition::Func(_) => {
                BTFuncCall {
                    func_index: defn,
                    args
                }
            },
            _ => {
                return Err(format!("procedure/code where function expected"));
            }
        })
    }
}
