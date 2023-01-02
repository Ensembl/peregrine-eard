/* All checking really involves is drawing up assignment-based equivalence-classes of registers
 * carrying a check and make sure all instances of a check are in the same equivalence class.
 * 
 * We require reduce to have run immediately before checking, so there are no register copies left
 * in the stream. So the only additional equivalences we need to add are those across code blocks.
 * As code blocks always innovate registers for their returns (we are still in the single-assignment
 * phase), we can just set any outputs to be equivalent to their inputs for the respective 
 * arguments.
 * 
 */

use std::{collections::{HashMap, HashSet}};
use crate::{frontend::{buildtree::{BuildTree, BTTopDefn}}, unbundle::linearize::Allocator, util::equiv::EquivalenceClass, model::{checkstypes::{CheckType}, linear::{LinearStatement, LinearStatementValue}, constants::Constant, codeblocks::CodeBlock}, controller::source::ParsePosition};
use super::broadtyping::BroadType;

pub(crate) struct Checking<'a> {
    bt: &'a BuildTree,
    position: ParsePosition,
    block_indexes: &'a HashMap<usize,usize>,
    equiv: HashMap<CheckType,EquivalenceClass<usize>>,
    group: HashMap<(CheckType,usize),HashSet<usize>>,
    forced: HashSet<usize>,
    check_register: HashMap<(CheckType,usize),usize>,
    allocator: &'a mut Allocator,
    broad: &'a mut HashMap<usize,BroadType>,
    out: Vec<LinearStatement>
}

impl<'a> Checking<'a> {
    pub(crate) fn new(bt: &'a BuildTree, block_indexes: &'a HashMap<usize,usize>, allocator: &'a mut Allocator, broad: &'a mut HashMap<usize,BroadType>) -> Checking<'a> {
        Checking {
            bt, block_indexes,
            position: ParsePosition::empty("called"),
            check_register: HashMap::new(),
            equiv: HashMap::new(),
            group: HashMap::new(),
            forced: HashSet::new(),
            allocator, broad,
            out: vec![]
        }
    }

    fn equiv(&mut self, ct: &CheckType) -> &mut EquivalenceClass<usize> {
        self.equiv.entry(ct.clone()).or_insert_with(|| EquivalenceClass::new())
    }

    fn equiv_block(&mut self, defn: &CodeBlock, rets: &[usize], args: &[usize]) -> Result<(),String> {
        let mut eq = HashMap::new();
        for (i,arg) in defn.arguments.iter().enumerate() {
            for check in &arg.checks {
                eq.entry(check.clone()).or_insert(vec![]).push((false,i));
            }
        }
        for (i,ret) in defn.results.iter().enumerate() {
            for check in &ret.checks {
                eq.entry(check.clone()).or_insert(vec![]).push((true,i));
            }
        }
        for (check,pos) in eq.drain() {
            let (ret_not_arg,index) = pos.first().unwrap();
            let regs = if *ret_not_arg { rets } else { args }; 
            let reg1 = regs[*index];
            for (ret_not_arg,index) in &pos {
                let src = if *ret_not_arg { rets } else { args }; 
                self.equiv(&check.check_type).equiv(reg1,src[*index]);
            }
        }
        Ok(())
    }

    fn make_equivs(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Copy(_, _) => { panic!("copy in checking: should have been run after reduce") }
            LinearStatementValue::Code(call,name,rets,args) => {
                let defn = match self.bt.get_by_index(*name)? {
                    BTTopDefn::Code(defn) => defn,
                    _ => { panic!("definition is not code definition"); }
                };
                let block_index = *self.block_indexes.get(call).expect("invalid block index");
                let defn = defn.get_block(block_index);
                self.equiv_block(defn,rets,args)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn done_making_equivs(&mut self) {
        for equiv in self.equiv.values_mut() {
            equiv.build();
        }
    }

    fn add_statement(&mut self, value: LinearStatementValue) {
        self.out.push(LinearStatement { value, position: self.position.clone() });
    }

    fn add_opcode(&mut self, special: &str, ret_regs: &[usize], arg_regs: &[usize]) {
        let name = self.bt.get_special(special);
        let call = self.allocator.next_call();
        self.add_statement(LinearStatementValue::Code(call,name,ret_regs.to_vec(),arg_regs.to_vec()));
    }

    fn add_check_code(&mut self, check_name: &str, check_fn: &str, ret_regs: Vec<usize>, a_reg: usize, b_reg: usize) {
        let checkname_reg = self.allocator.next_register();
        self.broad.insert(checkname_reg,BroadType::Atomic);
        let msg = format!("failed check of {} for {} at {:?}",check_name,check_fn,self.position);
        self.add_statement(LinearStatementValue::Constant(checkname_reg,Constant::String(msg.to_string())));
        self.add_opcode(check_fn,&ret_regs,&[checkname_reg,a_reg,b_reg]);
    }

    fn add_runtime_check(&mut self, reg: usize, check_name: &str, ct: &CheckType, ci: usize) {
        let value_reg = self.allocator.next_register();
        self.broad.insert(value_reg,BroadType::Atomic);
        let value_fn = match ct {
            CheckType::Length => "length",
            CheckType::LengthOrInfinite => "length",
            CheckType::Reference => "bound",
            CheckType::Sum => "total",
        };
        /* collect the parameter for this variable */
        self.add_opcode(value_fn,&[value_reg],&[reg]);
        /* verify our parameter is compatible with the check variable */
        match ct {
            CheckType::Length => {
                if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                    self.add_check_code(check_name,"check_length",vec![],value_reg,*existing);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Sum,ci)) {
                        self.add_check_code(check_name,"check_length_total",vec![],value_reg,*existing);
                    }
                    if let Some(existing) = self.check_register.get(&(CheckType::Reference,ci)) {
                        self.add_check_code(check_name,"check_length_bound",vec![],value_reg,*existing);
                    }
                    if let Some(existing) = self.check_register.get(&(CheckType::LengthOrInfinite,ci)) {
                        self.add_check_code(check_name,"check_length_inf",vec![],value_reg,*existing);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            },
            CheckType::Reference => {
                if let Some(_) = self.check_register.get(&(CheckType::Reference,ci)).cloned() {
                    let new_value_reg = self.allocator.next_register();
                    self.broad.insert(new_value_reg,BroadType::Atomic);
                    self.check_register.insert((ct.clone(),ci),new_value_reg);
                } else {
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
                /* always recheck bounds against length as bound (uniquely) can grow */
                if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                    self.add_check_code(check_name,"check_length_bound",vec![],*existing,value_reg);
                }
            },
            CheckType::Sum => {
                if let Some(existing) = self.check_register.get(&(CheckType::Sum,ci)) {
                    self.add_check_code(check_name,"check_total",vec![],value_reg,*existing);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                        self.add_check_code(check_name,"check_length_total",vec![],*existing,value_reg);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            },
            CheckType::LengthOrInfinite => {
                if let Some(existing) = self.check_register.get(&(CheckType::LengthOrInfinite,ci)) {
                    self.add_check_code(check_name,"check_inf",vec![],*existing,value_reg);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                        self.add_check_code(check_name,"check_length_inf",vec![],*existing,value_reg);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            }
        }
    }

    fn groupify(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Check(name,reg,ct,ci,force) => {
                let reg_group = *self.equiv(ct).get(reg);
                self.group.entry((ct.clone(),*ci)).or_insert_with(|| HashSet::new()).insert(reg_group);
                if *force {
                    self.forced.insert(reg_group);
                    self.add_runtime_check(*reg,name,ct,*ci);
                }
            },
            LinearStatementValue::Entry(_) => {
                self.check_register.clear();
                self.out.push(stmt.clone());
            },
            _ => {
                self.out.push(stmt.clone());
            }
        }
        Ok(())
    }

    /* We iterate again so that we get the line number in the error message */
    fn check(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Check(_,reg,ct,_,_) => {
                let group = *self.equiv(ct).get(reg);
                if !self.forced.contains(&group) {
                    return Err(format!("checking error: cannot guarantee {:?}",ct));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub(crate) fn run_checking(bt: &BuildTree, stmts: &[LinearStatement], block_indexes: &HashMap<usize,usize>, allocator: &mut Allocator,  broad: &mut HashMap<usize,BroadType>, verbose: bool) -> Result<Vec<LinearStatement>,String> {
    let mut typing = Checking::new(bt,block_indexes,allocator,broad);
    for stmt in stmts {
        typing.make_equivs(stmt).map_err(|e| typing.position.message(&e))?;
    }
    typing.done_making_equivs();
    for stmt in stmts {
        typing.groupify(stmt).map_err(|e| typing.position.message(&e))?;
    }
    for stmt in stmts {
        typing.check(stmt).map_err(|e| typing.position.message(&e))?;
    }
    if verbose {
        eprintln!("adding checks left {} statements",typing.out.len());
        typing.allocator.verbose();
    }
    Ok(typing.out.to_vec())
}
