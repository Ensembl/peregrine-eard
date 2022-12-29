use super::{operation::{OperationStore, Operation}, context::{ContextTemplateBuilder, ContextItem, RunContext}, globalcontext::GlobalBuildContext, program::ProgramStore, objectcode::{ObjectFile, Metadata, CompiledBlock}};


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

    pub fn load(&mut self, bytes: &[u8]) -> Result<(),String> {
        let block = ObjectFile::decode(bytes.to_vec()).map_err(|e| format!("loading error: {}",e))?;
        self.add(&block)?;
        Ok(())
    }

    pub(crate) fn add(&mut self, file: &ObjectFile) -> Result<(),String> {
        for code in &file.code {
            for (name,block) in &code.code {
                self.build(&code.metadata,name,block.clone())?;
            }
        }
        Ok(())
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
