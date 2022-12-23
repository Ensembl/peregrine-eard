use std::{collections::HashMap};
use crate::{middleend::narrowtyping::NarrowType, source::ParsePosition, unbundle::linearize::Allocator, model::{operation::{Operation, OperationValue}, constants::FullConstant}};

/* We have no main-store but can spill small constants as they can be regenerated. We force spills
 * after a certain non-reuse distance and allow patchup of non-spills during generation.
 */

/* When we come to want to use a register it can be in three states:
 * 1. Alive;
 * 2. a. Spilled and no longer alive;
 *    b. Spilled and then revifified one or more times but spilled again and no longer alive;
 * 3. Spilled and revivified into a new register.
 * 
 * revivification adds to "alias". If our source reg has an entry in alias, we are (2b) or (3) and
 * we call this the most-recent-name. Otherwise the most-recent-name is the original name. We look
 * up the most-recent-name in in_use. If it's there we can use that name (we were case (1) or (3)).
 * If absent, we are case (2). We look up the original name in "constants" and reviviy to put us in
 * case (3) and then use the new alias. Values not in constants were never constants.
 *
 */

const SPILL_IDLE : usize = 6;

struct Spill<'a> {
    in_use: HashMap<usize,Option<usize>>, // reg -> instr number for const regs still alive
    constants: HashMap<usize,FullConstant>, // reg -> value
    alias: HashMap<usize,usize>, // reg input -> reg output
    allocator: Allocator,
    narrow: &'a mut HashMap<usize,NarrowType>,
    out: Vec<Operation>
}

impl<'a> Spill<'a> {
    fn new(allocator: Allocator, narrow: &'a mut HashMap<usize,NarrowType>) -> Spill<'a> {
        Spill {
            in_use: HashMap::new(),
            constants: HashMap::new(),
            alias: HashMap::new(),
            allocator, narrow,
            out: vec![]
        }
    }

    fn remove_old(&mut self, index: usize) {
        let mut to_spill = vec![];
        for (reg,last_use) in &self.in_use {
            if let Some(last_use) = last_use {
                if last_use + SPILL_IDLE < index {
                    to_spill.push(*reg);
                }
            }
        }
        for reg in to_spill {
            self.in_use.remove(&reg);
        }
    }

    fn record_use(&mut self, index: usize, args: &[usize]) {
        for reg in args {
            if let Some(value) = self.in_use.get_mut(reg) {
                *value = Some(index);
            }
        }
    }

    fn vivify(&mut self, orig_reg: usize, position: &ParsePosition) -> usize {
        let reg = self.alias.get(&orig_reg).cloned().unwrap_or(orig_reg);
        if self.in_use.contains_key(&reg) {
            /* some version still alive */
            reg
        } else if let Some(value) = self.constants.get(&orig_reg) {
            /* vivify */
            let alias_reg = self.allocator.next_register();
            self.alias.insert(orig_reg,alias_reg);
            self.narrow.insert(alias_reg,self.narrow.get(&orig_reg).expect("missing type").clone());
            self.in_use.insert(alias_reg,None);
            self.out.push(Operation {
                position: position.clone(),
                value: OperationValue::Constant(alias_reg,value.clone())
            });
            alias_reg
        } else {
            /* not a constant */
            reg
        }
    }

    fn add(&mut self, index: usize, oper: &Operation) {
        self.remove_old(index);
        match &oper.value {
            OperationValue::Constant(reg,c) => {
                self.constants.insert(*reg,c.clone());
                self.in_use.insert(*reg,None);
                self.out.push(oper.clone());
            },
            OperationValue::Code(call,name,rets,args) => {
                let new_args = args.iter().map(|reg|
                    self.vivify(*reg,&oper.position)
                ).collect::<Vec<_>>();
                self.record_use(index,&new_args);
                self.out.push(Operation {
                    position: oper.position.clone(),
                    value: OperationValue::Code(*call,*name,rets.to_vec(),new_args)
                });
            },
            OperationValue::Entry(_) => {
                self.out.push(oper.clone());
            }
        }
    }

    fn take(self) -> Vec<Operation> { self.out }
}

pub(crate) fn spill(allocator: Allocator, opers: &[Operation], narrow: &mut HashMap<usize,NarrowType>) -> Vec<Operation> {
    let mut spill = Spill::new(allocator,narrow);
    for (i,oper) in opers.iter().enumerate() {
        spill.add(i,oper);
    }
    spill.take()
}
