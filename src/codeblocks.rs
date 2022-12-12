use std::fmt;
use crate::model::{CodeModifier, CodeBlock};


#[derive(Clone)]
pub struct CodeDefinition {
    world: bool,
    ret_count: Option<usize>,
    blocks: Vec<CodeBlock>
}

impl CodeDefinition {
    pub(crate) fn new() -> CodeDefinition {
        CodeDefinition {
            world: false,
            ret_count: None,
            blocks: vec![]
        }
    }

    pub(crate) fn add(&mut self, alt: CodeBlock) -> Result<(),String> {
        let ret_count = self.ret_count.get_or_insert(alt.results.len());
        if *ret_count != alt.results.len() {
            return Err(format!("code blocks must match in return value count: {} vs {}",alt.results.len(),ret_count));
        }
        if alt.modifiers.contains(&CodeModifier::World) {
            self.world = true;
        }
        self.blocks.push(alt);
        Ok(())
    }

    pub(crate) fn ret_count(&self) -> usize {
        self.ret_count.expect("query on empty code block")
    }

    pub(crate) fn world(&self) -> bool { self.world }
}

impl fmt::Debug for CodeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for block in &self.blocks {
            write!(f,"{:?}\n",block)?;
        }
        Ok(())
    }
}
