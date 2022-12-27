use std::collections::{HashSet, HashMap};
use crate::{model::operation::{Operation, OperationValue}, unbundle::linearize::Allocator, frontend::buildtree::{BuildTree, BTTopDefn}};

struct Large<'a> {
    allocator: &'a mut Allocator,
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    large_regs: HashSet<usize>,
    out: Vec<Operation>
}

impl<'a> Large<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, allocator: &'a mut Allocator) -> Large<'a> {
        Large {
            bt, block_index,
            allocator,
            large_regs: HashSet::new(),
            out: vec![]
        }
    }

    fn add_call_up(&mut self, oper: &Operation) -> Result<bool,String> {
        Ok(match &oper.value {
            OperationValue::Code(call,name,rets,args) => {
                let block_index = *self.block_index.get(&call).unwrap_or(&0);
                let block = match self.bt.get_by_index(*name)? {
                    BTTopDefn::Code(c) => c.get_block(block_index),
                    _ => { panic!("didn't get code with code index"); }
                };
                let args_large = args.iter().map(|r| self.large_regs.contains(r)).collect::<Vec<_>>();
                let (large_instr,large_regs) = block.is_large(&args_large)?;
                for reg in &large_regs {
                    self.large_regs.insert(rets[*reg]);
                }
                large_instr
            },
            _ => false
        })
    }

    fn add(&mut self, oper: &Operation) -> Result<(),String> {
        self.out.push(oper.clone());
        if self.add_call_up(oper)? {
            let name = self.bt.get_special("libcore__call_up");
            let call = self.allocator.next_call();
            self.out.push(Operation {
                position: oper.position.clone(),
                value: OperationValue::Code(call,name,vec![],vec![])
            });
        }
        Ok(())
    }
}

pub(crate) fn large(bt: &BuildTree, block_index: &HashMap<usize,usize>,allocator: &mut Allocator, opers: &[Operation]) -> Result<Vec<Operation>,String> {
    let mut large = Large::new(bt,block_index,allocator);
    for oper in opers {
        large.add(oper)?;
    }
    Ok(large.out)
}
