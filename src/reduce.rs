use std::collections::HashMap;

use crate::model::{LinearStatement, LinearStatementValue};

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
            LinearStatementValue::Check(reg,ct,ci) => {
                Some(LinearStatementValue::Check(self.canon(*reg),ct.clone(),*ci))
            },
            LinearStatementValue::Constant(reg,c) => {
                Some(LinearStatementValue::Constant(self.canon(*reg),c.clone()))
            },
            LinearStatementValue::Type(reg,typ) => {
                Some(LinearStatementValue::Type(self.canon(*reg),typ.clone()))
            }
            LinearStatementValue::Code(index,name,rets,args,world) => {
                let rets = rets.iter().map(|reg| self.canon(*reg)).collect::<Vec<_>>();
                let args = args.iter().map(|reg| self.canon(*reg)).collect::<Vec<_>>();
                Some(LinearStatementValue::Code(*index,*name,rets,args,*world))
            },
            LinearStatementValue::Copy(dst,src) => {
                let src = self.equiv.get(src).unwrap_or(src);
                self.equiv.insert(*dst,*src);
                None
            }
        };
        value.map(|value| {
            let mut out = stmt.clone();
            out.value = value;
            out
        })
    }
}

pub(crate) fn reduce(stmts: &[LinearStatement]) -> Vec<LinearStatement> {
    let mut reduce = Reduce::new();
    stmts.iter().filter_map(|f| reduce.reduce(f)).collect()
}
