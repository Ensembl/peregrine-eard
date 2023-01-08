use std::{sync::Arc, mem, collections::HashMap};
use super::{objectcode::ProgramName, operation::{OperationStore, Step}, context::RunContext, globalcontext::{GlobalContext, GlobalBuildContext}, value::Value};

pub struct ProgramStore {
    store: OperationStore,
    program: HashMap<(ProgramName,String),Program>
}

impl ProgramStore {
    pub(crate) fn new(store: OperationStore) -> ProgramStore {
        ProgramStore { store, program: HashMap::new() }
    }

    pub(crate) fn program_builder(&self, step_by_step: bool) -> ProgramBuilder {
        ProgramBuilder::new(&self.store,step_by_step)
    }

    pub(crate) fn add_program(&mut self, metadata: &ProgramName, block: &str, program: Program) {
        self.program.insert((metadata.clone(),block.to_string()),program);
    }

    pub(crate) fn list_programs(&self) -> Vec<ProgramName> {
        self.program.keys().map(|(m,_)| m.clone()).collect()
    }

    pub(crate) fn list_blocks(&self, metadata: &ProgramName) -> Vec<String> {
        self.program.keys().filter_map(|(m,b)| {
            if m == metadata { Some(b.to_string()) } else { None }
        }).collect()
    }

    pub(crate) fn get(&self, metadata: &ProgramName, block: &str) -> Result<&Program,String> {
        self.program
            .get(&(metadata.clone(),block.to_string()))
            .ok_or_else(|| format!("no such program {:?} {:?}",metadata,block))
    }
}

#[derive(Clone)]
pub struct Program {
    max_reg: usize,
    steps: Arc<Vec<Step>>,
    constants: Arc<Vec<Value>>,
    step_details: Arc<Vec<(usize,Vec<usize>)>>
}

impl Program {
    async fn run_step_by_step(&self, context: RunContext) -> Result<(),String> {
        let mut gctx = GlobalContext::new(self.max_reg,&self.constants,context);
        for (i,step) in self.steps.as_ref().iter().enumerate() {
            eprintln!("\n\n{:?}",self.step_details[i]);
            for reg in &self.step_details[i].1 {
                eprintln!("  before r{} = {:?}",*reg,gctx.get(*reg));
            }
            step.run(&mut gctx).await?;
            eprintln!("");
            for reg in &self.step_details[i].1 {
                eprintln!("  after  r{} = {:?}",*reg,gctx.get(*reg));
            }
        }
        Ok(())
    }

    async fn run_fast(&self, context: RunContext) -> Result<(),String> {
        let mut gctx = GlobalContext::new(self.max_reg,&self.constants,context);
        for step in self.steps.as_ref().iter() {
            step.run(&mut gctx).await?;
        }
        Ok(())
    }

    pub async fn run(&self, context: RunContext) -> Result<(),String> {
        if self.step_details.len() > 0 {
            self.run_step_by_step(context).await
        } else {
            self.run_fast(context).await
        }
    }
}

pub struct ProgramBuilder<'a> {
    symbols: bool,
    max_reg: usize,
    store: &'a OperationStore,
    constants: Vec<Value>,
    steps: Vec<Step>,
    step_details: Vec<(usize,Vec<usize>)>  
}

impl<'a> ProgramBuilder<'a> {
    pub(crate) fn new(store: &'a OperationStore, symbols: bool) -> ProgramBuilder<'a> {
        ProgramBuilder { 
            store, constants: vec![], steps: vec![], max_reg: 0, step_details: vec![],
            symbols
        }
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
        if self.symbols {
            self.step_details.push((opcode,registers.clone()));
        }
        self.steps.push(Step::new(gbctx,oper,registers)?);
        Ok(())
    }

    pub(crate) fn to_program(&mut self) -> Program {
        Program {
            max_reg: self.max_reg,
            steps: Arc::new(mem::replace(&mut self.steps,vec![])),
            constants: Arc::new(mem::replace(&mut self.constants,vec![])),
            step_details: Arc::new(mem::replace(&mut self.step_details,vec![]))
        }
    }
}
