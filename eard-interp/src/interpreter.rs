use crate::{operation::{OperationStore, self, Operation}, program::{Program, ProgramStore}, objectcode::{CompiledBlock, Metadata}, globalcontext::GlobalBuildContext, context::{ContextTemplateBuilder, ContextItem, RunContext}};

pub struct InterpreterBuilder {
    pub context: ContextTemplateBuilder,
    pub store: OperationStore
}

impl InterpreterBuilder {
    pub fn new() ->  InterpreterBuilder {
        InterpreterBuilder {
            context: ContextTemplateBuilder::new(),
            store: OperationStore::new()
        }
    }

    pub fn add_context<T: 'static>(&mut self, name: &str) -> ContextItem<T> {
        self.context.add(name)
    }

    pub fn add_operation(&mut self, opcode: usize, oper: Operation) {
        self.store.add(opcode,oper);
    }
}

pub struct Interpreter {
    gbctx: GlobalBuildContext,
    store: ProgramStore
}

impl Interpreter {
    pub fn new(builder: InterpreterBuilder) -> Interpreter {
        Interpreter { 
            gbctx: GlobalBuildContext::new(builder.context),
            store: ProgramStore::new(builder.store)
        }
    }

    pub(crate) fn build(&mut self, metadata: &Metadata, block: &str, code: CompiledBlock) -> Result<(),String> {
        let mut builder = self.store.program_builder();
        let program = code.to_program(&self.gbctx,&mut builder)?;
        self.store.add_program(metadata,block,program);
        Ok(())
    }

    pub async fn run(&self, metadata: &Metadata, block: &str, context: RunContext) -> Result<(),String> {
        self.store.run(metadata,block,context).await
    }
}
