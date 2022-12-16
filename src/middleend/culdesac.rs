use std::{collections::{HashMap, HashSet}, sync::Arc, mem};
use crate::{model::{Operation, CodeModifier, OperationValue}, frontend::buildtree::{BuildTree, BTTopDefn}, codeblocks::CodeBlock};

struct CulDeSac<'a> {
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    roots: HashSet<usize>,
    requires: HashMap<usize,Arc<Vec<usize>>>,
    needed: HashSet<usize>,
    worlds: HashSet<usize>
}

impl<'a> CulDeSac<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>) -> CulDeSac<'a> {
        CulDeSac { 
            bt, block_index,
            roots: HashSet::new(),
            requires: HashMap::new(),
            needed: HashSet::new(),
            worlds: HashSet::new()
        }
    }

    fn get_block(&self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_index.get(&call).expect("missing block index");
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn add_oper(&mut self, oper: &Operation) {
        match &oper.value {
            OperationValue::Constant(_, _) => {},
            OperationValue::Code(call,name,rets,args) => {
                let block = self.get_block(*call,*name);
                if block.modifiers.contains(&CodeModifier::World) {
                    self.worlds.insert(*call);
                    for arg in args {
                        self.roots.insert(*arg);
                    }
                }
                let args = Arc::new(args.to_vec());
                for ret in rets {
                    self.requires.insert(*ret,args.clone());
                }
            },
        }
    }

    fn iterate(&mut self, reg: usize) {
        if self.needed.contains(&reg) { return; }
        self.needed.insert(reg);
        if let Some(kids) = self.requires.get(&reg).clone() {
            for kid in kids.clone().as_ref().iter() {
                self.iterate(*kid);
            }            
        }
    }

    fn iterate_roots(&mut self) {
        for root in mem::replace(&mut self.roots,HashSet::new()) {
            self.iterate(root);
        }
    }

    fn include(&mut self, oper: &Operation) -> Option<Operation> {
        let value = match &oper.value {
            OperationValue::Constant(reg,c) => {
                if self.needed.contains(reg) {
                    Some(OperationValue::Constant(*reg,c.clone()))
                } else {
                    None
                }
            },
            OperationValue::Code(call,name,dst,src) => {
                let regs_needed = dst.iter().any(|reg| self.needed.contains(reg));
                if !regs_needed && !self.worlds.contains(call) { return None; }
                let dsts = dst.iter().map(|reg| {
                    if self.needed.contains(reg) { *reg } else { 0 }
                }).collect::<Vec<_>>();
                Some(OperationValue::Code(*call,*name,dsts,src.to_vec()))
            },
        };
        value.map(|value| Operation {
            position: oper.position.clone(),
            value
        })
    }
}

pub(crate) fn culdesac(bt: &BuildTree, block_index: &HashMap<usize,usize>, opers: &[Operation]) -> Vec<Operation> {
    let mut culdesac = CulDeSac::new(bt,block_index);
    for oper in opers {
        culdesac.add_oper(oper);
    }
    culdesac.iterate_roots();
    opers.iter().filter_map(|op| culdesac.include(op)).collect()
}
