use std::{future::Future, pin::Pin};

use super::globalcontext::{GlobalBuildContext, GlobalContext};

pub enum Return {
    Sync,
    Async(Pin<Box<dyn Future<Output = Result<(),String>>>>)
}

pub struct OperationStore {
    opers: Vec<Operation>
}

impl OperationStore {
    pub(crate) fn new() -> OperationStore {
        OperationStore { opers: vec![] }
    }

    pub fn add(&mut self, opcode: usize, operation: Operation) {
        if self.opers.len() <= opcode {
            self.opers.resize_with(opcode+1,|| Operation::nop());
        }
        self.opers[opcode] = operation;
    }

    pub(crate) fn get(&self, opcode: usize) -> Result<&Operation,String> {
        self.opers.get(opcode).ok_or_else(|| format!("no such opcode {}",opcode))
    }
}

pub struct Operation {
    callback: Box<dyn Fn(&GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext, &[usize]) -> Result<Return,String>>,String>>
}

impl Operation {
    pub fn new<F>(callback: F) -> Operation
            where F: Fn(&GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext, &[usize]) -> Result<Return,String>>,String> + 'static {
        Operation { callback: Box::new(callback) }
    }

    pub fn nop() -> Operation {
        Operation::new(|_| Ok(Box::new(|_,_| Ok(Return::Sync))))
    }

    fn make(&self, gbctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext, &[usize]) -> Result<Return,String>>,String> {
        (self.callback)(gbctx)
    }
}

pub(crate) struct Step {
    callback: Box<dyn Fn(&mut GlobalContext, &[usize]) -> Result<Return,String>>,
    pub(crate) registers: Vec<usize>
}

impl Step {
    pub(crate) fn new(gbctx: &GlobalBuildContext, operation: &Operation, registers: Vec<usize>) -> Result<Step,String> {
        Ok(Step { callback: operation.make(gbctx)?, registers })
    }

    pub(crate) async fn run(&self, gctx: &mut GlobalContext) -> Result<(),String> {
        match (self.callback)(gctx,&self.registers)? {
            Return::Sync => Ok(()),
            Return::Async(w) => w.await,
        }
    }
}

