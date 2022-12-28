use std::sync::Arc;

use crate::{handles::{HandleStore}, value::Value, context::{ContextTemplate, ContextTemplateBuilder, RunContext}};

pub struct GlobalBuildContext {
    pub patterns: ContextTemplate
}

impl GlobalBuildContext {
    pub(crate) fn new(mut patterns: ContextTemplateBuilder) -> GlobalBuildContext {
        GlobalBuildContext { patterns: patterns.build() }
    }
}

pub struct GlobalContext {
    pub registers: HandleStore<Value>,
    pub constants: Arc<Vec<Value>>,
    pub context: RunContext
}

impl GlobalContext {
    pub(crate) fn new(max_reg: usize, constants: &Arc<Vec<Value>>, context: RunContext) -> GlobalContext {
        let mut regs = HandleStore::new();
        regs.init(max_reg,Value::default());
        GlobalContext {
            registers: regs, constants: constants.clone(), context
        }
    }
}
