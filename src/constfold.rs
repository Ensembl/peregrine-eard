use std::{sync::Arc, collections::{HashMap}};
use crate::{model::{LinearStatement, Operation, LinearStatementValue, FullConstant, CodeModifier}, compilation::EarpCompilation, frontend::buildtree::{BuildTree, BTTopDefn}, codeblocks::CodeBlock};

struct ConstFold<'a,'b> {
    comp: &'b EarpCompilation<'a>,
    bt: &'b BuildTree,
    block_indexes: &'b HashMap<usize,usize>,
    values: HashMap<usize,FullConstant>,
    position: Option<(Arc<Vec<String>>,usize)>,
    out: Vec<Operation>
}

impl<'a,'b> ConstFold<'a,'b> {
    fn new(compilation: &'b EarpCompilation<'a>, bt: &'b BuildTree, block_indexes: &'b HashMap<usize,usize>) -> ConstFold<'a,'b> {
        ConstFold {
            comp: compilation, bt, block_indexes,
            values: HashMap::new(),
            position: None,
            out: vec![]
        }
    }

    fn get_block(&mut self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_indexes.get(&call).expect("missing block index");
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn fold(&mut self, name: &str, rets: &[usize], args: &[usize]) -> bool {
        let inputs = args.iter().map(|a| self.values.get(a).cloned()).collect::<Vec<_>>();
        if let Some(outputs) = self.comp.compiler().fold(name,&inputs) {
            for (reg,c) in rets.iter().zip(outputs.iter()) {
                self.values.insert(*reg,c.clone());
                self.out.push(Operation::Constant(*reg,c.clone()));
            }
            true
        } else {
            false
        }
    }

    fn code(&mut self, block: &CodeBlock, name: usize, rets: &[usize], args: &[usize]) {
        let folds = block.modifiers.iter().filter_map(|m| {
            match m {
                CodeModifier::Fold(f) => Some(f.to_string()),
                _ => None
            }
        }).collect::<Vec<_>>();
        for fold in &folds {
            if self.fold(fold,rets,args) { return; }
        }
        self.out.push(Operation::Code(name,rets.to_vec(),args.to_vec()));
    }

    fn add_line_no(&mut self, stmt: &LinearStatement) {
        let mut add = true;
        if let Some(old_position) = &self.position {
            if old_position.0 == stmt.file && old_position.1 == stmt.line_no {
                add = false;
            }
        }
        if add {
            self.out.push(Operation::Line(stmt.file.clone(),stmt.line_no));
            self.position = Some((stmt.file.clone(),stmt.line_no));
        }
    }

    fn add(&mut self, stmt: &LinearStatement) {
        self.add_line_no(stmt);
        match &stmt.value {
            LinearStatementValue::Check(_, _, _, _) => {},
            LinearStatementValue::Constant(reg,c) => {
                self.out.push(Operation::Constant(*reg,FullConstant::Atomic(c.clone())));
                self.values.insert(*reg,FullConstant::Atomic(c.clone()));
            },
            LinearStatementValue::Copy(_, _) => { panic!("copy occurred in constfold") },
            LinearStatementValue::Code(call,name,rets,args) => {
                let block = self.get_block(*call,*name).clone();
                self.code(&block,*name,rets,args);
            },
            LinearStatementValue::Type(_, _) => {},
            LinearStatementValue::WildEquiv(_) => {},
        }
    }

    fn take(self) -> Vec<Operation> { self.out }
}

pub(crate) fn const_fold<'a,'b>(compilation: &'b EarpCompilation<'a>, bt: &'b BuildTree, block_indexes: &'b HashMap<usize,usize>, stmts: &[LinearStatement]) -> Vec<Operation> {
    let mut fold = ConstFold::new(compilation,bt,block_indexes);
    for stmt in stmts {
        fold.add(stmt);
    }
    fold.take()
}
