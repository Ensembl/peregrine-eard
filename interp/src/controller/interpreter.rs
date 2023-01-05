use crate::ProgramName;
use super::{operation::{OperationStore, Operation}, context::{ContextTemplateBuilder, ContextItem, RunContext}, globalcontext::GlobalBuildContext, program::{ProgramStore, Program}, objectcode::{ObjectFile, CompiledBlock, Metadata }, version::OpcodeVersion};

pub struct InterpreterBuilder {
    version: OpcodeVersion,
    context: ContextTemplateBuilder,
    store: OperationStore
}

impl InterpreterBuilder {
    pub fn new() ->  InterpreterBuilder {
        let mut out = InterpreterBuilder {
            version: OpcodeVersion::new(),
            context: ContextTemplateBuilder::new(),
            store: OperationStore::new()
        };
        out.add_version("core",(0,0));
        out
    }

    pub fn add_version(&mut self, name: &str, version: (u32,u32)) {
        self.version.add_version(name,version);
    }

    pub fn add_context<T: 'static>(&mut self, name: &str) -> ContextItem<T> {
        self.context.add(name)
    }

    pub fn add_operation(&mut self, opcode: usize, oper: Operation) {
        self.store.add(opcode,oper);
    }
}

pub struct Interpreter {
    version: OpcodeVersion,
    gbctx: GlobalBuildContext,
    store: ProgramStore
}

impl Interpreter {
    pub fn new(builder: InterpreterBuilder) -> Interpreter {
        Interpreter { 
            version: builder.version,
            gbctx: GlobalBuildContext::new(builder.context),
            store: ProgramStore::new(builder.store)
        }
    }

    pub fn list_programs(&self) -> Vec<ProgramName> {
        self.store.list_programs()
    }

    pub fn list_blocks(&self, metadata: &ProgramName) -> Vec<String> {
        self.store.list_blocks(metadata)
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
        metadata.version.meets_minimums(&self.version)?;
        let mut builder = self.store.program_builder();
        let program = code.to_program(&self.gbctx,&mut builder)?;
        self.store.add_program(&metadata.name,block,program);
        Ok(())
    }

    pub fn get(&self, metadata: &ProgramName, block: &str) -> Result<&Program,String> {
        self.store.get(metadata,block)
    }
}
