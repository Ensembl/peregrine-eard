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

use std::{sync::Arc, collections::HashMap};
use crate::{frontend::{parsetree::at, buildtree::{BuildTree, BTTopDefn}}, model::{LinearStatement, LinearStatementValue, CheckType}, codeblocks::{CodeDefinition, CodeBlock}, equiv::EquivalenceClass};

pub(crate) struct Checking<'a> {
    bt: &'a BuildTree,
    position: Option<(Arc<Vec<String>>,usize)>,
    block_indexes: &'a HashMap<usize,usize>,
    equiv: HashMap<CheckType,EquivalenceClass<usize>>,
    group: HashMap<(CheckType,usize),usize>
}

impl<'a> Checking<'a> {
    pub(crate) fn new(bt: &'a BuildTree, block_indexes: &'a HashMap<usize,usize>) -> Checking<'a> {
        Checking {
            bt, block_indexes,
            position: None,
            equiv: HashMap::new(),
            group: HashMap::new()
        }
    }

    fn error_at(&self, msg: &str) -> String {
        self.position.as_ref().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
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
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            LinearStatementValue::Copy(_, _) => { panic!("copy in checking: should have been run after reduce") }
            LinearStatementValue::Code(call,name,rets,args,world) => {
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

    fn check(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            LinearStatementValue::Check(reg,ct,ci) => {
                let reg_group = *self.equiv(ct).get(reg);
                if let Some(old_group) = self.group.get(&(ct.clone(),*ci)).cloned() {
                    if reg_group != old_group {
                        return Err(format!("checking error: cannot guarantee {:?}",ct));
                    }
                } else {
                    self.group.insert((ct.clone(),*ci),reg_group);
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub(crate) fn run_checking(bt: &BuildTree, stmts: &[LinearStatement], block_indexes: &HashMap<usize,usize>) -> Result<(),String> {
    let mut typing = Checking::new(bt,block_indexes);
    for stmt in stmts {
        typing.make_equivs(stmt).map_err(|e| typing.error_at(&e))?;
    }
    typing.done_making_equivs();
    for stmt in stmts {
        typing.check(stmt).map_err(|e| typing.error_at(&e))?;
    }
    Ok(())
}