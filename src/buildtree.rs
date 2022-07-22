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

    pub(crate) fn define(&mut self, name: &str, definition: BTDefinition, location: (&Arc<Vec<String>>,usize)) -> Result<(),String> {
        let id = self.definitions.len();
        self.defnames.insert(name.to_string(),id);
        self.definitions.push(definition);
        let stmt = BTStatement {
            value: BTStatementValue::Define(id),
            file: location.0.clone(),
            line_no: location.1
        };
        self.statements.push(stmt);
        Ok(())
    }

    pub(crate) fn declare(&mut self, declaration: BTDeclare, location: (&Arc<Vec<String>>,usize)) -> Result<(),String> {
        let stmt = BTStatement {
            value: BTStatementValue::Declare(declaration),
            file: location.0.clone(),
            line_no: location.1
        };
        self.statements.push(stmt);
        Ok(())
    }

    pub(crate) fn check(&mut self, variable: &Variable, check: &Check, location: (&Arc<Vec<String>>,usize)) -> Result<(),String> {
        let stmt = BTStatement {
            value: BTStatementValue::Check(variable.clone(),check.clone()),
            file: location.0.clone(),
            line_no: location.1
        };
        self.statements.push(stmt);
        Ok(())
    }
}
