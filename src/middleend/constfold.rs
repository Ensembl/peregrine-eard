use std::{sync::Arc, collections::{HashMap}};
use crate::{model::{LinearStatement, Operation, LinearStatementValue, FullConstant, CodeModifier, OperationValue}, compilation::EarpCompilation, frontend::buildtree::{BuildTree, BTTopDefn}, codeblocks::CodeBlock};

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

    fn out(&mut self, value: OperationValue) {
        let position = if let Some(pos) = &self.position {
            pos.clone()
        } else {
            (Arc::new(vec!["*anon*".to_string()]),0)
        };
        self.out.push(Operation { position, value });
    }

    fn fold(&mut self, name: &str, rets: &[usize], args: &[usize]) -> bool {
        let inputs = args.iter().map(|a| self.values.get(a).cloned()).collect::<Vec<_>>();
        if let Some(outputs) = self.comp.compiler().fold(name,&inputs) {
            for (reg,c) in rets.iter().zip(outputs.iter()) {
                self.values.insert(*reg,c.clone());
                self.out(OperationValue::Constant(*reg,c.clone()));
            }
            true
        } else {
            false
        }
    }

    fn code(&mut self, block: &CodeBlock, call: usize, name: usize, rets: &[usize], args: &[usize]) {
        let folds = block.modifiers.iter().filter_map(|m| {
            match m {
                CodeModifier::Fold(f) => Some(f.to_string()),
                _ => None
            }
        }).collect::<Vec<_>>();
        for fold in &folds {
            if self.fold(fold,rets,args) { return; }
        }
        self.out(OperationValue::Code(call,name,rets.to_vec(),args.to_vec()));
    }

    fn add(&mut self, stmt: &LinearStatement) {
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            LinearStatementValue::Check(_, _, _, _) => {},
            LinearStatementValue::Constant(reg,c) => {
                self.out(OperationValue::Constant(*reg,FullConstant::Atomic(c.clone())));
                self.values.insert(*reg,FullConstant::Atomic(c.clone()));
            },
            LinearStatementValue::Copy(_, _) => { panic!("copy occurred in constfold") },
            LinearStatementValue::Code(call,name,rets,args) => {
                let block = self.get_block(*call,*name).clone();
                self.code(&block,*call,*name,rets,args);
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
