use std::{sync::Arc};

use crate::{model::{ Variable, Check, Constant, ArgTypeSpec, OrBundle, TypedArgument, CodeBlock, OrBundleRepeater}};

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
    Register(usize)
}

#[derive(Debug,Clone)]
pub struct BTProcCall<R> {
    pub(crate) proc_index: Option<usize>,
    pub(crate) args: Vec<OrBundleRepeater<BTExpression>>,
    pub(crate) rets: Option<Vec<R>>
}

#[derive(Debug,Clone)]
pub struct BTFuncCall {
    pub(crate) func_index: usize,
    pub(crate) args: Vec<OrBundleRepeater<BTExpression>>
}

#[derive(Debug,Clone)]
pub enum BTStatementValue {
    Define(usize),
    Declare(OrBundleRepeater<Variable>),
    Check(Variable,Check),
    BundledStatement(BTProcCall<OrBundleRepeater<BTLValue>>),
    UnbundledStatement(BTProcCall<Vec<BTLValue>>)
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
                c.add(block.clone());
            },
            _ => {
                panic!("Incorrectly indexed definition");
            }
        }
        Ok(())
    }

    pub(crate) fn statement(&mut self, defn: Option<usize>, args: Vec<OrBundleRepeater<BTExpression>>, rets: Option<Vec<OrBundleRepeater<BTLValue>>>) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::BundledStatement(BTProcCall {
            proc_index: defn,
            args, rets
        }))
    }

    pub(crate) fn function_call(&self, defn: usize, args: Vec<OrBundleRepeater<BTExpression>>) -> Result<BTFuncCall,String> {
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

    pub(crate) fn get_function(&self, f: &BTFuncCall) -> Result<&BTFuncProcDefinition,String> {
        Ok(match &self.definitions[f.func_index] {
            BTDefinition::Func(f) => f,
            _ => { return Err(format!("expected function, got non-function")); }
        })
    }

    pub(crate) fn get_procedure(&self, p: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Option<&BTFuncProcDefinition>,String> {
        if let Some(index) = p.proc_index {
            Ok(match &self.definitions[index] {
                BTDefinition::Proc(p) => Some(p),
                _ => { return Err(format!("expected function, got non-function")); }
            })
        } else {
            Ok(None)
        }
    }
}
