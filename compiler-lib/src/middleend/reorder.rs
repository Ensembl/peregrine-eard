/*
 * We reorder to allow maximal use of modifying opcodes for our code blocks. This is a toposort
 * problem. The dag has two node types: one for every instruction (instruction nodes) and
 * one which represents all uses of a register but for one having taken place (last chance node).
 * For example, if register X is used in instructions A, B, C, there is a last chance node (X,A) 
 * which is placed after B and C, a node (X,B) after A and C and a node (X,C) after A and B.
 * 
 * In reality, last chance nodes are created only when needed but for analysis can be treated as
 * always present. There are the following edges.
 * 
 * 1. For each register a "register edge" is added from the instruction which generated the
 * register to each instruction which uses it.
 * 
 * 2. For the chain of world instructions, an edge is added in existing program order, a "world
 * edge".
 * 
 * 3. Every time a register is used as an argument, a "last chance edge" is added to all relevant
 * last chance nodes. (In reality, lazily on creation of the last chance node).
 * 
 * These edges all function identically, their name is just used to indicated their origin.
 * Together, this graph represents all acceptable orderings of instructions.
 * 
 * When an instruction exists in a modifying form, at least one pair of registers can be
 * identified, the "source" register which would be overritten and the "destination" register
 * which would take the new value. If we can add an edge from the last chance node of the source
 * register for this instruction, to the instruction in which the destination register is created
 * without adding cycles, we can exploit the modfying form and reuse the register as the last
 * chance node guarantees this is the last use of the register.
 * 
 * We toposort the initial graph and then use an incremental algorithm to add a node A->B. We 
 * limit complexity by putting a limit on the number of nodes searched. World edges are linear
 * and last chance edges convergent, so we would only expect branching from register edges. That is,
 * the complex searches would occur when a register is used in many places. Such registers may not
 * end up being optimised due to time constraints.
 */

use std::{collections::HashMap, mem};
use crate::{frontend::{buildtree::{BTTopDefn, BuildTree}}, util::toposort::TopoSort, model::{operation::{Operation, OperationValue}, codeblocks::{CodeModifier, CodeBlock}, constants::OperationConstant}, controller::source::ParsePosition};

#[derive(PartialEq,Eq,Hash,Clone,Debug)]
enum ReorderNode {
    LastChance(usize,usize), // all uses of reg .0 except instr .1
    Instruction(usize)
}

struct Reorder<'a> {
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    topo: TopoSort<ReorderNode>,
    reg_birth: HashMap<usize,usize>,
    worlds: Vec<usize>,
    constants: HashMap<usize,OperationConstant>,
    uses: HashMap<usize,Vec<usize>>,
    useful_arcs: Vec<(ReorderNode,ReorderNode)>, // src -> dst
    position: ParsePosition,
    credit: u64
}

impl<'a> Reorder<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, credit: u64) -> Reorder<'a> {
        Reorder {
            bt, block_index,
            topo: TopoSort::new(),
            reg_birth: HashMap::new(),
            worlds: vec![],
            uses: HashMap::new(),
            constants: HashMap::new(),
            useful_arcs: vec![],
            position: ParsePosition::empty("called"),
            credit
        }
    }

    fn get_block(&self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_index.get(&call).unwrap_or(&0);
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn add_nodes(&mut self, index: usize, oper: &Operation) {
        match &oper.value {
            OperationValue::Constant(reg,c) => {
                self.topo.node(ReorderNode::Instruction(index),None);
                self.reg_birth.insert(*reg,index);
                self.constants.insert(*reg,c.clone());
            },
            OperationValue::Code(call,name,rets,args) => {
                for reg in args {
                    self.uses.entry(*reg).or_insert(vec![]).push(index);
                }
                for reg in rets {
                    self.reg_birth.insert(*reg,index);
                }
                self.topo.node(ReorderNode::Instruction(index),None);
                let block = self.get_block(*call,*name);
                if block.modifiers.contains(&CodeModifier::World) {
                    self.worlds.push(index);
                }
            },
            OperationValue::Entry(_) => {
                self.topo.node(ReorderNode::Instruction(index),None);
                self.worlds.push(index);
            }
        }
    }

    fn finish_arcs(&mut self) {
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
        match &oper.value {
            OperationValue::Constant(_,_) => {},
            OperationValue::Entry(_) => {},
            OperationValue::Code(_,_,_,args) => {
                for arg in args {
                    /* register edge */
                    self.topo.arc(
                        &ReorderNode::Instruction(*self.reg_birth.get(arg).expect("use of unknown register")),
                        &ReorderNode::Instruction(index)
                    );
                }
            }
        }
    }

    fn build(&mut self) {
        self.topo.sort();
    }

    fn make_useful_arcs(&mut self, index: usize, oper: &Operation) -> Result<(),String> {
        self.position = oper.position.clone();
        match &oper.value {
            OperationValue::Constant(_, _) => {},
            OperationValue::Entry(_) => {},
            OperationValue::Code(call,name,_,args) => {
                let block = self.get_block(*call,*name);
                let mut useful = vec![];
                let inputs = args.iter().map(|a| self.constants.get(a).map(|x| x.to_full_constant())).collect::<Vec<_>>();
                let imps = block.choose_imps(&inputs,None,None);
                for imp in imps {
                    for (_,arg_pos) in imp.reg_reuse()? {
                        useful.push(args[arg_pos]);
                    }
                }
                for reg in useful.drain(..) {
                    self.topo.node(ReorderNode::LastChance(reg,index),Some(&ReorderNode::Instruction(index)));
                    for other_use in self.uses.get(&reg).unwrap_or(&vec![]) {
                        if *other_use != index {
                            self.topo.arc(&ReorderNode::Instruction(*other_use),
                                &ReorderNode::LastChance(reg,index));
                        }
                    }
                    self.useful_arcs.push((
                        ReorderNode::LastChance(reg,index),
                        ReorderNode::Instruction(index)
                    ));
                }
            }
        }
        Ok(())
    }

    fn add_useful_arcs(&mut self) {
        let mut useful = mem::replace(&mut self.useful_arcs, vec![]);
        useful.sort_by_key(|(a,b)| {
            self.topo.distance(&a,&b)
        });
        for (src,dst) in useful.drain(..) {
            let distance = self.topo.distance(&src,&dst).unwrap_or(0) as u64;
            if distance > self.credit { return; }
            self.credit -= distance;
            self.topo.arc(&src,&dst);
        }
    }
}

pub(crate) fn reorder(bt: &BuildTree, block_index: &HashMap<usize,usize>, opers: &[Operation]) -> Result<Vec<Operation>,String> {
    /* populate and toposort initial graph */
    let mut reorder = Reorder::new(bt,block_index,10_000_000);
    for (i,oper) in opers.iter().enumerate() {
        reorder.add_nodes(i,oper);
    }
    reorder.finish_arcs();
    for (i,oper) in opers.iter().enumerate() {
        reorder.add_main_arcs(i,oper);
    }
    reorder.build();
    /* find instructions where we'd like to attempt a modifiable form */
    for (i,oper) in opers.iter().enumerate() {
        reorder.make_useful_arcs(i,oper).map_err(|e| reorder.position.message(&e))?;
    }
    reorder.add_useful_arcs();
    let order = reorder.topo.order().unwrap().iter().filter_map(|x| match x {
        ReorderNode::LastChance(_, _) => None,
        ReorderNode::Instruction(i) => Some(*i),
    }).collect::<Vec<_>>();
    let opers = order.iter().map(|idx| opers[*idx].clone()).collect::<Vec<_>>();
    Ok(opers.to_vec())
}
