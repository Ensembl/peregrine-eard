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
use crate::{frontend::{buildtree::{BuildTree, BTTopDefn}}, model::{LinearStatement, LinearStatementValue, CheckType, Constant, AtomicTypeSpec}, codeblocks::{CodeBlock}, equiv::{EquivalenceClass}, source::ParsePosition, unbundle::linearize::Allocator};

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

    fn add_check_code(&mut self, check_name: &str, check_fn: &str, a_reg: usize, b_reg: usize) {
        let checkname_reg = self.allocator.next_register();
        self.broad.insert(checkname_reg,BroadType::Atomic(AtomicTypeSpec::String));
        self.out.push(LinearStatement { 
            value: LinearStatementValue::Constant(checkname_reg,Constant::String(check_name.to_string())),
            position: self.position.clone()
        });
        let name = self.bt.get_special(&check_fn);
        let call = self.allocator.next_call();
        self.out.push(LinearStatement {
            value: LinearStatementValue::Code(call,name,vec![],vec![checkname_reg,a_reg,b_reg]),
            position: self.position.clone(),
        });
    }

    fn add_runtime_check(&mut self, reg: usize, check_name: &str, ct: &CheckType, ci: usize) {
        let value_reg = self.allocator.next_register();
        self.broad.insert(value_reg,BroadType::Atomic(AtomicTypeSpec::Number));
        let value_fn = match ct {
            CheckType::Length => "length",
            CheckType::LengthOrInfinite => "length",
            CheckType::Reference => "bound",
            CheckType::Sum => "total",
        };
        let name = self.bt.get_special(value_fn);
        let call = self.allocator.next_call();
        self.out.push(LinearStatement {
            value: LinearStatementValue::Code(call,name,vec![value_reg],vec![reg]),
            position: self.position.clone(),
        });
        match ct {
            CheckType::Length => {
                if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                    self.add_check_code(check_name,"check_length",value_reg,*existing);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Sum,ci)) {
                        self.add_check_code(check_name,"check_length_total",value_reg,*existing);
                    }
                    if let Some(existing) = self.check_register.get(&(CheckType::Reference,ci)) {
                        self.add_check_code(check_name,"check_length_bound",value_reg,*existing);
                    }
                    if let Some(existing) = self.check_register.get(&(CheckType::LengthOrInfinite,ci)) {
                        self.add_check_code(check_name,"check_length_inf",value_reg,*existing);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            },
            CheckType::Reference => {
                if let Some(existing) = self.check_register.get(&(CheckType::Reference,ci)) {
                    self.add_check_code(check_name,"check_bound",value_reg,*existing);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                        self.add_check_code(check_name,"check_length_bound",*existing,value_reg);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            },
            CheckType::Sum => {
                if let Some(existing) = self.check_register.get(&(CheckType::Sum,ci)) {
                    self.add_check_code(check_name,"check_total",value_reg,*existing);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                        self.add_check_code(check_name,"check_length_total",*existing,value_reg);
                    }
                    self.check_register.insert((ct.clone(),ci),value_reg);
                }
            },
            CheckType::LengthOrInfinite => {
                if let Some(existing) = self.check_register.get(&(CheckType::LengthOrInfinite,ci)) {
                    self.add_check_code(check_name,"check_inf",*existing,value_reg);
                } else {
                    if let Some(existing) = self.check_register.get(&(CheckType::Length,ci)) {
                        self.add_check_code(check_name,"check_length_inf",*existing,value_reg);
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

pub(crate) fn run_checking(bt: &BuildTree, stmts: &[LinearStatement], block_indexes: &HashMap<usize,usize>, allocator: &mut Allocator,  broad: &mut HashMap<usize,BroadType>) -> Result<Vec<LinearStatement>,String> {
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
    Ok(typing.out.to_vec())
}
