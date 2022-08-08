use std::{sync::Arc};

use crate::{model::{ Variable, Check, CallArg, Constant, ArgTypeSpec, OrBundle, TypedArgument, CodeBlock}};

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
    pub(crate) blocks: Vec<CodeBlock>
}

impl BTCodeDefinition {
    fn add(&mut self, alt: CodeBlock) {
        self.blocks.push(alt);
    }
}

#[derive(Debug,Clone)]
pub struct BTFuncProcDefinition {
    pub(crate) args: Vec<OrBundle<TypedArgument>>,
    pub(crate) captures: Vec<OrBundle<Variable>>,
    pub(crate) block: Vec<BTStatement>,
    pub(crate) ret: Vec<OrBundle<BTExpression>>,
    pub(crate) ret_type: Option<Vec<ArgTypeSpec>>
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

    pub(super) fn add_definition(&mut self, defn: BTDefinition) -> usize {
        let id = self.definitions.len();
        self.definitions.push(defn);
        id
    }

    pub(crate) fn add_code(&mut self, id: usize, block: &CodeBlock) -> Result<(),String> {
        match &mut self.definitions[id] {
            BTDefinition::Code(c) => {
                c.add(block.clone());
            },
            _ => {
                panic!("Incorrectly indexed definition");
            }
        }
        Ok(())
    }

    pub(crate) fn statement(&mut self, defn: Option<usize>, args: Vec<CallArg<BTExpression>>, rets: Option<Vec<BTLValue>>) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Statement(BTProcCall {
            proc_index: defn,
            args, rets
        }))
    }

    pub(crate) fn function_call(&self, defn: usize, args: Vec<CallArg<BTExpression>>) -> Result<BTFuncCall,String> {
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

    pub(super) fn add_statement(&mut self, stmt: BTStatement) {
        self.statements.push(stmt);
    }
}
