use std::{sync::Arc, collections::{HashMap, BTreeMap}};

use crate::model::{CodeModifier, Variable, Check};

#[derive(Debug,Clone)]
pub struct BTLetAssign {

}

#[derive(Debug,Clone)]
pub struct BTExpression {

}

#[derive(Debug,Clone)]
pub struct BTCall {
    proc_index: Option<usize>
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
    Statement(BTCall)
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
    location: (Arc<Vec<String>>,usize)
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext { location: (Arc::new(vec!["*anon*".to_string()]),0) }
    }

    pub fn set_location(&mut self, file: &Arc<Vec<String>>, line_no: usize) {
        self.location = (file.clone(),line_no);
    }

    pub fn location(&self) -> (&[String],usize) {
        (self.location.0.as_ref(),self.location.1)
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
    defnames: BTreeMap<String,usize>,
    definitions: Vec<BTDefinition>
}

impl BuildTree {
    pub(crate) fn new() -> BuildTree {
        BuildTree { statements: vec![], defnames: BTreeMap::new(), definitions: vec![] }
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

    pub(crate) fn define(&mut self, name: &str, definition: BTDefinition, bc: &BuildContext) -> Result<(),String> {
        let id = self.definitions.len();
        self.defnames.insert(name.to_string(),id);
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
}
