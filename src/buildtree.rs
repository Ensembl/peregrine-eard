use std::{sync::Arc, collections::{HashMap, BTreeMap}};

use crate::model::{CodeModifier, Variable, Check, CallArg, Constant};

#[derive(Debug,Clone)]
pub struct BTLetAssign {

}

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

pub struct BuildContext {
    location: (Arc<Vec<String>>,usize),
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),usize>,
    next_register: usize
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext {
            location: (Arc::new(vec!["*anon*".to_string()]),0),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0
        }
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
}

#[derive(Debug,Clone)]
pub enum BTDefinition {
    Code(BTCodeDefinition),
    Func(),
    Proc()
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

    fn add_statement(&mut self, value: BTStatementValue, bc: &BuildContext) -> Result<(),String> {
        let stmt = BTStatement {
            value,
            file: bc.location.0.clone(),
            line_no: bc.location.1
        };
        self.statements.push(stmt);
        Ok(())
    }

    pub(crate) fn define(&mut self, name: &str, definition: BTDefinition, bc: &mut BuildContext, export: bool) -> Result<(),String> {
        let id = self.definitions.len();
        let context = if export { None } else { Some(bc.file_context) };
        bc.defnames.insert((context,name.to_string()),id);
        self.definitions.push(definition);
        self.add_statement(BTStatementValue::Define(id),bc)?;
        Ok(())
    }

    pub(crate) fn declare(&mut self, declaration: BTDeclare, bc: &BuildContext) -> Result<(),String> {
        self.add_statement(BTStatementValue::Declare(declaration),bc)?;
        Ok(())
    }

    pub(crate) fn check(&mut self, variable: &Variable, check: &Check, bc: &BuildContext) -> Result<(),String> {
        self.add_statement(BTStatementValue::Check(variable.clone(),check.clone()),bc)?;
        Ok(())
    }

    pub(crate) fn statement(&mut self, defn: Option<usize>, args: Vec<CallArg<BTExpression>>, rets: Option<Vec<BTLValue>>, bc: &BuildContext) -> Result<(),String> {
        let call = match defn.map(|x| &self.definitions[x]) {
            Some(BTDefinition::Code(_)) => {
                todo!()
            },
            Some(BTDefinition::Func()) => {
                BTProcCall {
                    proc_index: None,
                    args, rets
                }
            },
            Some(BTDefinition::Proc()) => {
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
        self.add_statement(BTStatementValue::Statement(call),bc)?;
        Ok(())
    }

    pub(crate) fn function_call(&self, defn: usize, args: Vec<CallArg<BTExpression>>, bc: &BuildContext) -> Result<BTFuncCall,String> {
        Ok(match &self.definitions[defn] {
            BTDefinition::Func() => {
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
