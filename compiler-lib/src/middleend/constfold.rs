use std::{collections::{HashMap}};
use crate::{frontend::buildtree::{BuildTree, BTTopDefn}, model::{constants::{FullConstant, OperationConstant}, operation::{Operation, OperationValue}, linear::{LinearStatementValue, LinearStatement}, codeblocks::{CodeBlock, CodeModifier}, checkstypes::AtomicTypeSpec}, controller::{compilation::EardCompilation, source::ParsePosition}};
use super::narrowtyping::NarrowType;

struct ConstFold<'a,'b> {
    comp: &'b EardCompilation<'a>,
    bt: &'b BuildTree,
    narrow: &'b HashMap<usize,NarrowType>,
    block_indexes: &'b HashMap<usize,usize>,
    values: HashMap<usize,FullConstant>,
    position: ParsePosition,
    out: Vec<Operation>
}

impl<'a,'b> ConstFold<'a,'b> {
    fn new(compilation: &'b EardCompilation<'a>, bt: &'b BuildTree, block_indexes: &'b HashMap<usize,usize>, narrow: &'b HashMap<usize,NarrowType>) -> ConstFold<'a,'b> {
        ConstFold {
            comp: compilation, bt, block_indexes, narrow,
            values: HashMap::new(),
            position: ParsePosition::empty("called"),
            out: vec![]
        }
    }

    fn get_block(&mut self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_indexes.get(&call).unwrap_or(&0);
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn out(&mut self, value: OperationValue) {
        self.out.push(Operation { position: self.position.clone(), value });
    }

    fn to_constant(&mut self, reg: usize, c: FullConstant) {
        let mut empty_list = false;
        if let FullConstant::Finite(a) = &c {
            if a.len() == 0 { empty_list = true; }
        }
        let value = if empty_list {
            let n = match self.narrow.get(&reg).expect("missing register") {
                NarrowType::Sequence(n) => n,
                _ => { panic!("contradictory typing in constfold") }
            };
            match n {
                AtomicTypeSpec::Number => OperationConstant::EmptyNumberSeq,
                AtomicTypeSpec::String => OperationConstant::EmptyStringSeq,
                AtomicTypeSpec::Boolean => OperationConstant::EmptyBooleanSeq,
                AtomicTypeSpec::Handle(c) => OperationConstant::EmptyHandleSeq(c.to_string())
            }
        } else {
            OperationConstant::Constant(c)
        };
        self.out(OperationValue::Constant(reg,value));
    }

    fn fold(&mut self, name: &str, rets: &[usize], args: &[usize]) -> bool {
        let inputs = args.iter().map(|a| self.values.get(a).cloned()).collect::<Vec<_>>();
        if let Some(outputs) = self.comp.compiler().fold(name,&inputs) {
            for (reg,c) in rets.iter().zip(outputs.iter()) {
                self.values.insert(*reg,c.clone());
                self.to_constant(*reg,c.clone());
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
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Check(_,_,_,_,_) => {},
            LinearStatementValue::Constant(reg,c) => {
                self.out(OperationValue::Constant(*reg,OperationConstant::Constant(FullConstant::Atomic(c.clone()))));
                self.values.insert(*reg,FullConstant::Atomic(c.clone()));
            },
            LinearStatementValue::Copy(_, _) => { panic!("copy occurred in constfold") },
            LinearStatementValue::Code(call,name,rets,args) => {
                let block = self.get_block(*call,*name).clone();
                self.code(&block,*call,*name,rets,args);
            },
            LinearStatementValue::Signature(_) => {},
            LinearStatementValue::Entry(s) => {
                self.values.clear();
                self.out(OperationValue::Entry(s.to_string()));
            },
        }
    }

    fn take(self) -> Vec<Operation> { self.out }
}

pub(crate) fn const_fold<'a,'b>(compilation: &'b EardCompilation<'a>, bt: &'b BuildTree, block_indexes: &'b HashMap<usize,usize>, narrow: &HashMap<usize,NarrowType>, stmts: &[LinearStatement], verbose: bool) -> Vec<Operation> {
    let mut fold = ConstFold::new(compilation,bt,block_indexes,narrow);
    for stmt in stmts {
        fold.add(stmt);
    }
    if verbose {
        eprintln!("folding constants left {} statements",fold.out.len());
    }
    fold.take()
}
