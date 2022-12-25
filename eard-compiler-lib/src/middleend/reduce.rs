use std::collections::{HashMap, HashSet};

use crate::model::{linear::{LinearStatementValue, LinearStatement}, checkstypes::{TypeRestriction, intersect_restrictions}};

/* NOTE! After linearizing we are not yet in signle-assignment form as multiple consecutive calls
 * to a function/procedure reuse registers (we can get away without rewriting or a stack because)
 * we are non-recursive. It takes a call to reduce to convert to single-assignment form. This is
 * also why reduce() is not an EquivelenceClass as the semantics are slightly different. (We are
 * also lazy during sequence generation and reuse registers but this could be changed were it
 * necessary that we are not in SAF anyway).
 */

struct Reduce {
    equiv: HashMap<usize,usize>
}

impl Reduce {
    fn new() -> Reduce {
        Reduce {
            equiv: HashMap::new()
        }
    }

    fn canon(&self, reg: usize) -> usize {
        self.equiv.get(&reg).cloned().unwrap_or(reg)
    }

    fn merge_restr_set(&mut self, mut restr_set: Vec<Vec<TypeRestriction>>) -> Result<Vec<TypeRestriction>,String> {
        if let Some(mut restr) = restr_set.pop() {
            let mut out = restr.drain(..).collect::<HashSet<_>>();
            for mut another in restr_set {
                let more = another.drain(..).collect::<HashSet<_>>();
                out = intersect_restrictions(&out,&more);
            }
            Ok(out.drain().collect())
        } else {
            Ok(vec![])
        }
    }

    fn reduce(&mut self, stmt: &LinearStatement) -> Result<Option<LinearStatement>,String> {
        let value = match &stmt.value {
            LinearStatementValue::Check(name,reg,ct,ci,f) => {
                Some(LinearStatementValue::Check(name.clone(),self.canon(*reg),ct.clone(),*ci,*f))
            },
            LinearStatementValue::SameType(regs) => {
                let mut regs = regs.iter().map(|r| self.canon(*r)).collect::<Vec<_>>();
                regs.sort();
                regs.dedup();
                if regs.len() > 1 {
                    Some(LinearStatementValue::SameType(regs))
                } else {
                    None
                }
            },
            LinearStatementValue::Signature(s) => {
                let s = s.iter().map(|(reg,spec)| {
                    (self.canon(*reg),spec.to_vec())
                }).collect::<Vec<_>>();
                Some(LinearStatementValue::Signature(s))
            },
            LinearStatementValue::Constant(reg,c) => {
                Some(LinearStatementValue::Constant(self.canon(*reg),c.clone()))
            },
            LinearStatementValue::Type(reg,typ) => {
                Some(LinearStatementValue::Type(self.canon(*reg),typ.clone()))
            }
            LinearStatementValue::Code(index,name,rets,args) => {
                let rets = rets.iter().map(|reg| self.canon(*reg)).collect::<Vec<_>>();
                let args = args.iter().map(|reg| self.canon(*reg)).collect::<Vec<_>>();
                Some(LinearStatementValue::Code(*index,*name,rets,args))
            },
            LinearStatementValue::Copy(dst,src) => {
                let src = self.equiv.get(src).unwrap_or(src);
                self.equiv.insert(*dst,*src);
                None
            },
            LinearStatementValue::Entry(s) => { 
                Some(LinearStatementValue::Entry(s.to_string()))
            }
        };
        Ok(value.map(|value| {
            let mut out = stmt.clone();
            out.value = value;
            out
        }))
    }
}

pub(crate) fn reduce(stmts: &[LinearStatement], verbose: bool) -> Result<Vec<LinearStatement>,String> {
    let mut reduce = Reduce::new();
    let mut out = vec![];
    for stmt in stmts {
        if let Some(new_stmt) = reduce.reduce(stmt)? {
            out.push(new_stmt);
        }
    }
    if verbose {
        eprintln!("reduced to {} statements by removing copies",out.len());
    }
    Ok(out)
}
