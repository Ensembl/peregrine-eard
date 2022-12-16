/*
 * We reorder to allow maximal use of modifying opcodes for our code blocks. THis is a toposort
 * problem. The dag has two node types: one for every instruction (instruction nodes) and one for
 * "after the last use of a register" (tombstone nodes).
 * 
 * 1. For each register a "register edge" is added from the instruction which generated the
 * register to each instruction which uses it.
 * 
 * 2. For the chain of world instructions, an edge is added in existing program order, a "world
 * edge".
 * 
 * 3. Every time a register is used as an argument, a "tombstone edge" is added from the
 * instruction node to the tombstone node.
 * 
 * These edges all function identically, their name is just used to indicated their origin.
 * Together, this graph represents all acceptable orderings of instructions.
 * 
 * When an instruction exists in a modifying form, at least one pair of registers can be
 * identified, the "source" register which would be overritten and the "destination" register
 * which would take the new value. If we can add an edge from the tombstone node of the source
 * register to the instruction in which the destination register is created without adding cycles,
 * we can exploit the modfying form and reuse the register.
 * 
 * We toposort the initial graph and then use an incremental algorithm to add a node A->B. We 
 * limit complexity by putting a limit on the number of nodes searched. World edges are linear
 * and tombstone edges convergent, so we would only expect branching from register edges. That is,
 * the complex searches would occur when a register is used in many places. Such registers may not
 * end up being optimised due to time constraints.
 */

use std::collections::HashMap;
use crate::{toposort::TopoSort, model::{Operation, CodeModifier}, frontend::buildtree::{BTTopDefn, BuildTree}, codeblocks::CodeBlock};

#[derive(PartialEq,Eq,Hash,Clone)]
enum ReorderNode {
    Tombstone(usize),
    Instruction(usize)
}

struct Reorder<'a> {
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    topo: TopoSort<ReorderNode>,
    reg_birth: HashMap<usize,usize>,
    worlds: Vec<usize>
}

impl<'a> Reorder<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, limit: u32) -> Reorder<'a> {
        Reorder {
            bt, block_index,
            topo: TopoSort::new(Some(limit)),
            reg_birth: HashMap::new(),
            worlds: vec![]
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

    fn add_nodes(&mut self, index: usize, oper: &Operation) {
        match oper {
            Operation::Line(_, _) => {},
            Operation::Constant(reg, _) => {
                self.topo.node(ReorderNode::Tombstone(*reg));
                self.topo.node(ReorderNode::Instruction(index));
                self.reg_birth.insert(*reg,index);
            },
            Operation::Code(call,name,rets, _) => {
                for reg in rets {
                    self.topo.node(ReorderNode::Tombstone(*reg));
                    self.reg_birth.insert(*reg,index);
                }
                self.topo.node(ReorderNode::Instruction(index));
                let block = self.get_block(*call,*name);
                if block.modifiers.contains(&CodeModifier::World) {
                    self.worlds.push(index);
                }
            },
        }
    }

    fn add_world_arcs(&mut self) {
        self.worlds.reverse();
        if let Some(mut prev) = self.worlds.pop() {
            while let Some(next) = self.worlds.pop() {
                self.topo.arc(
                    &ReorderNode::Instruction(prev),
                    &ReorderNode::Instruction(next)
                );
                prev = next;
            }
        }
    }

    fn add_main_arcs(&mut self, index: usize, oper: &Operation) {
        match oper {
            Operation::Line(_,_) => {},
            Operation::Constant(_,_) => {}
            Operation::Code(_,_,_,args) => {
                for arg in args {
                    /* register edge */
                    self.topo.arc(
                        &ReorderNode::Instruction(*self.reg_birth.get(arg).expect("use of unknown register")),
                        &ReorderNode::Instruction(index)
                    );
                    /* tombstone edge */
                    self.topo.arc(
                        &ReorderNode::Instruction(index),
                        &ReorderNode::Tombstone(*arg)
                    );
                }
            }
        }
    }

    fn build(&mut self) {
        self.topo.sort();
    }
}

pub(crate) fn reorder(bt: &BuildTree, block_index: &HashMap<usize,usize>, opers: &[Operation]) -> Vec<Operation> {
    let limit = 10_000_000 / (opers.len()+1);
    let mut reorder = Reorder::new(bt,block_index,limit as u32);
    for (i,oper) in opers.iter().enumerate() {
        reorder.add_nodes(i,oper);
    }
    reorder.add_world_arcs();
    for (i,oper) in opers.iter().enumerate() {
        reorder.add_main_arcs(i,oper);
    }
    reorder.build();
    opers.to_vec()
}
