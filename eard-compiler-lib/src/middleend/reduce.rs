use std::collections::HashMap;

use crate::model::linear::{LinearStatementValue, LinearStatement};

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

    fn reduce(&mut self, stmt: &LinearStatement) -> Option<LinearStatement> {
        let value = match &stmt.value {
            LinearStatementValue::Check(name,reg,ct,ci,f) => {
                Some(LinearStatementValue::Check(name.clone(),self.canon(*reg),ct.clone(),*ci,*f))
            },
            LinearStatementValue::WildEquiv(regs) => {
                let mut regs = regs.iter().map(|r| self.canon(*r)).collect::<Vec<_>>();
                regs.sort();
                regs.dedup();
                if regs.len() > 1 {
                    Some(LinearStatementValue::WildEquiv(regs))
                } else {
                    None
                }
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
        value.map(|value| {
            let mut out = stmt.clone();
            out.value = value;
            out
        })
    }
}

pub(crate) fn reduce(stmts: &[LinearStatement], verbose: bool) -> Vec<LinearStatement> {
    let mut reduce = Reduce::new();
    let out = stmts.iter().filter_map(|f| reduce.reduce(f)).collect::<Vec<_>>();
    if verbose {
        eprintln!("reduced to {} statements by removing copies",out.len());
    }
    out
}
