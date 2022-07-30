use std::{sync::Arc, collections::{HashMap, BTreeMap}};

use crate::model::{CodeModifier, Variable, Check, CallArg, Constant, ArgTypeSpec, OrBundle, TypedArgument};

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
    pub modifiers: Vec<CodeModifier>
}

impl BTCodeDefinition {
    pub(crate) fn new(modifiers: Vec<CodeModifier>) -> BTCodeDefinition {
        BTCodeDefinition {
            modifiers
        }
    }
}

struct CurrentDefinition {
    block: Vec<BTStatement>,
    name: String,
    export: bool,
    args: Vec<OrBundle<TypedArgument>>, // empty for code 
    variety: BTDefinitionVariety,
    ret_type: Option<Vec<ArgTypeSpec>>, // exactly one for functions, None for code
    code_modifiers: Vec<CodeModifier>, // code only
}

impl CurrentDefinition {
    fn to_funcproc(&self, ret: &[OrBundle<BTExpression>]) -> BTFuncProcDefinition {
        BTFuncProcDefinition {
            args: self.args.clone(),
            ret_type: self.ret_type.clone(),
            ret: ret.to_vec(),
            block: self.block.clone()
        }
    }

    fn to_code(&self) -> BTCodeDefinition {
        BTCodeDefinition {
            modifiers: self.code_modifiers.clone()
        }
    }
}

pub struct BuildContext {
    location: (Arc<Vec<String>>,usize),
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),usize>,
    next_register: usize,
    code_target: Option<CurrentDefinition>
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext {
            location: (Arc::new(vec!["*anon*".to_string()]),0),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0,
            code_target: None
        }
    }

    pub fn push_funcproc_target(&mut self, variety: BTDefinitionVariety, name: &str,
            args: &[OrBundle<TypedArgument>],
            ret_type: Option<Vec<ArgTypeSpec>>,
            export: bool) {
        self.code_target = Some(CurrentDefinition {
            name: name.to_string(),
            args: args.to_vec(),
            export, variety,
            block: vec![],
            ret_type,
            code_modifiers: vec![]
        });
    }

    pub fn push_code_target(&mut self, name: &str, 
            modifiers: Vec<CodeModifier>) {
        self.code_target = Some(CurrentDefinition {
            name: name.to_string(),
            args: vec![],
            export: false,
            variety: BTDefinitionVariety::Code,
            block: vec![],
            ret_type: None,
            code_modifiers: modifiers
        });
    }

    pub fn pop_target(&mut self, ret: &[OrBundle<BTExpression>], bt: &mut BuildTree) -> Result<(),String> {
        let ctx = self.code_target.take().expect("pop without push");
        let defn = match &ctx.variety {
            BTDefinitionVariety::Code => BTDefinition::Code(ctx.to_code()),
            BTDefinitionVariety::Func => BTDefinition::Func(ctx.to_funcproc(ret)),
            BTDefinitionVariety::Proc => BTDefinition::Proc(ctx.to_funcproc(ret))
        };
        bt.define(&ctx.name,defn,self,ctx.export)?;
        self.code_target = None;
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

    pub(crate) fn lookup(&self, name: &str) -> Result<usize,String> {
        self.defnames
            .get(&(Some(self.file_context),name.to_string()))
            .or_else(||
                self.defnames.get(&(None,name.to_string()))
            )
            .ok_or_else(|| format!("No such function/procedure {}",name))
            .cloned()
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
        if let Some(target) = self.code_target.as_mut() {
            target.block.push(stmt);
        } else {
            bt.statements.push(stmt);
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct BTFuncProcDefinition {
    pub args: Vec<OrBundle<TypedArgument>>,
    block: Vec<BTStatement>,
    ret: Vec<OrBundle<BTExpression>>,
    pub(crate) ret_type: Option<Vec<ArgTypeSpec>>
}

pub enum BTDefinitionVariety { Code, Func, Proc }

#[derive(Debug,Clone)]
pub enum BTDefinition {
    Code(BTCodeDefinition),
    Func(BTFuncProcDefinition),
    Proc(BTFuncProcDefinition)
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

    fn define(&mut self, name: &str, definition: BTDefinition, bc: &mut BuildContext, export: bool) -> Result<BTStatementValue,String> {
        let id = self.definitions.len();
        let context = if export { None } else { Some(bc.file_context) };
        bc.defnames.insert((context,name.to_string()),id);
        self.definitions.push(definition);
        Ok(BTStatementValue::Define(id))
    }

    pub(crate) fn declare(&mut self, declaration: BTDeclare, bc: &BuildContext) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Declare(declaration))
    }

    pub(crate) fn check(&mut self, variable: &Variable, check: &Check, bc: &BuildContext) -> Result<BTStatementValue,String> {
        Ok(BTStatementValue::Check(variable.clone(),check.clone()))
    }

    pub(crate) fn statement(&mut self, defn: Option<usize>, args: Vec<CallArg<BTExpression>>, rets: Option<Vec<BTLValue>>, bc: &BuildContext) -> Result<BTStatementValue,String> {
        let call = match defn.map(|x| &self.definitions[x]) {
            Some(BTDefinition::Code(_)) => {
                todo!()
            },
            Some(BTDefinition::Func(_)) => {
                BTProcCall {
                    proc_index: None,
                    args, rets
                }
            },
            Some(BTDefinition::Proc(_)) => {
                BTProcCall {
                    proc_index: defn,
                    args, rets
                }
            },
            None => {
                BTProcCall {
                    proc_index: None,
                    args, rets
                }
            }
        };
        Ok(BTStatementValue::Statement(call))
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
