use std::{future::Future, pin::Pin, sync::{Arc, Mutex}};

use super::globalcontext::{GlobalBuildContext, GlobalContext};

pub struct AsyncReturnImpl<T> {
    async_part: Arc<Mutex<Pin<Box<dyn Future<Output = Result<T,String>>>>>>,
    sync_part: Box<dyn Fn(&mut GlobalContext,&[usize],T) -> Result<(),String>>
}

trait ErasedAsyncReturn {
    fn callback<'a>(&'a self, gctx: &'a mut GlobalContext, regs: &'a [usize]) -> Pin<Box<dyn Future<Output = Result<(),String>> +'a>>;
}

impl<T: 'static> ErasedAsyncReturn for AsyncReturnImpl<T> {
    fn callback<'a>(&'a self, gctx: &'a mut GlobalContext, regs: &'a [usize]) -> Pin<Box<dyn Future<Output = Result<(),String>> + 'a>> {
        let async_part = self.async_part.clone();
        Box::pin(async move {
            /* Each time the initial, sync part of op is run, it returns a new async so we can't
             * get mutex clashes.
             */
            let v = async_part.lock().unwrap().as_mut().await?;
            (self.sync_part)(gctx,regs,v)
        })
    }
}

pub struct AsyncReturn(Box<dyn ErasedAsyncReturn>);

impl AsyncReturn {
    pub fn new<T: 'static,F>(async_part: Pin<Box<dyn Future<Output = Result<T,String>>>>,
               sync_part: F) -> AsyncReturn
            where F: Fn(&mut GlobalContext,&[usize],T) -> Result<(),String> + 'static {
        let out = AsyncReturnImpl { 
            async_part: Arc::new(Mutex::new(async_part)),
            sync_part: Box::new(sync_part)
        };
        AsyncReturn(Box::new(out))
    }

    pub(crate) fn callback<'a>(&'a self, gctx: &'a mut GlobalContext, regs: &'a [usize]) -> Pin<Box<dyn Future<Output = Result<(),String>> + 'a>> {
        self.0.callback(gctx,regs)
    }
}

pub enum Return {
    Sync,
    Async(AsyncReturn)
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
            Return::Async(ear) => ear.callback(gctx,&self.registers).await
        }
    }
}

