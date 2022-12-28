use std::{sync::Arc, mem, collections::HashMap};
use crate::{operation::{Step, OperationStore}, value::Value, globalcontext::{GlobalContext, GlobalBuildContext}, objectcode::Metadata, context::RunContext};

pub struct ProgramStore {
    store: OperationStore,
    program: HashMap<(Metadata,String),Program>
}

impl ProgramStore {
    pub(crate) fn new(store: OperationStore) -> ProgramStore {
        ProgramStore { store, program: HashMap::new() }
    }

    pub(crate) fn store(&self) -> &OperationStore { &self.store }

    pub(crate) fn program_builder(&self) -> ProgramBuilder {
        ProgramBuilder::new(&self.store)
    }

    pub(crate) fn add_program(&mut self, metadata: &Metadata, block: &str, program: Program) {
        self.program.insert((metadata.clone(),block.to_string()),program);
    }

    pub(crate) async fn run(&self, metadata: &Metadata, block: &str, context: RunContext) -> Result<(),String> {
        let program = self.program
            .get(&(metadata.clone(),block.to_string()))
            .ok_or_else(|| format!("no such program {:?} {:?}",metadata,block))?;
        program.run(context).await;
        Ok(())
    }
}

pub struct Program {
    max_reg: usize,
    steps: Arc<Vec<Step>>,
    constants: Arc<Vec<Value>>
}

impl Program {
    async fn run(&self, context: RunContext) {
        let mut gctx = GlobalContext::new(self.max_reg,&self.constants,context);
        for step in self.steps.as_ref().iter() {
            step.run(&mut gctx).await;
        }
    }
}

pub struct ProgramBuilder<'a> {
    max_reg: usize,
    store: &'a OperationStore,
    constants: Vec<Value>,
    steps: Vec<Step>    
}

impl<'a> ProgramBuilder<'a> {
    pub(crate) fn new(store: &'a OperationStore) -> ProgramBuilder<'a> {
        ProgramBuilder { store, constants: vec![], steps: vec![], max_reg: 0 }
    }

    pub(crate) fn add_constant(&mut self, index: usize, value: Value) {
        if self.constants.len() <= index {
            self.constants.resize(index+1,Value::default());
        }
        self.constants[index] = value;
    }

    pub(crate) fn add_opcode(&mut self, gbctx: &GlobalBuildContext, opcode: usize, registers: Vec<usize>) -> Result<(),String> {
        self.max_reg = self.max_reg.max(*registers.iter().max().unwrap_or(&0));
        let oper = self.store.get(opcode)?;
        self.steps.push(Step::new(gbctx,oper,registers)?);
        Ok(())
    }

    pub(crate) fn to_program(&mut self) -> Program {
        Program {
            max_reg: self.max_reg,
            steps: Arc::new(mem::replace(&mut self.steps,vec![])),
            constants: Arc::new(mem::replace(&mut self.constants,vec![]))
        }
    }
}
