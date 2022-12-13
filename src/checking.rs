use std::{sync::Arc, collections::HashMap};
use crate::{frontend::{parsetree::at, buildtree::BuildTree}, model::{LinearStatement, LinearStatementValue, CheckType}};

struct Equiv {
    regs: HashMap<usize,usize>, // reg -> index
    equiv: HashMap<usize,usize> // index -> index
}

impl Equiv {
    fn new() -> Equiv {
        Equiv {
            regs: HashMap::new(),
            equiv: HashMap::new()
        }
    }

    fn set_reg(&mut self, reg: usize, index: usize) {
        if let Some(known_index) = self.regs.get(&reg) {
            self.set_equiv_index(*known_index,index);
        } else {
            self.regs.insert(reg,index);
        }
    }

    fn canon(&self, mut val: usize) -> usize {
        while let Some(new_val) = self.equiv.get(&val) {
            val = *new_val;
        }
        val
    }

    fn set_equiv_index(&mut self, a: usize, b: usize) {
        let a = self.canon(a);
        let b = self.canon(b);
        if a != b {
            self.equiv.insert(a,b);
        }
    }

    fn set_equiv(&mut self, reg_a: usize, reg_b: usize) {
        match (self.regs.get(&reg_a),self.regs.get(&reg_b)) {
            (None, None) => {},
            (None, Some(_)) => todo!(),
            (Some(_), None) => todo!(),
            (Some(_), Some(_)) => todo!(),
        }

        if let Some(a) = self.regs.get(&reg_a) {
            if let Some(b) = self.regs.get(&reg_b) {
                self.set_equiv_index(*a,*b);
            } else {
                self.regs.insert(reg_b,*a);
            }
        } else {

        }
    }
}

pub(crate) struct Checking<'a> {
    bt: &'a BuildTree,
    position: Option<(Arc<Vec<String>>,usize)>,
    equiv: HashMap<CheckType,Equiv>
}

impl<'a> Checking<'a> {
    pub(crate) fn new(bt: &'a BuildTree) -> Checking<'a> {
        Checking {
            bt,
            position: None,
            equiv: HashMap::new()
        }
    }

    fn equiv(&mut self, ct: &CheckType) -> &mut Equiv {
        self.equiv.entry(ct.clone()).or_insert_with(|| Equiv::new())
    }

    fn set_equiv(&mut self, dst: usize, src: usize) {
        for equiv in self.equiv.values_mut() {
            equiv.set_equiv(dst,src);
        }
    }

    fn error_at(&self, msg: &str) -> String {
        self.position.as_ref().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
    }

    fn add(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = Some((stmt.file.clone(),stmt.line_no));
        todo!();
        Ok(())
    }
}

pub(crate) fn run_checking(bt: &BuildTree, stmts: &[LinearStatement]) -> Result<(),String> {
    let mut typing = Checking::new(bt);
    for stmt in stmts {
        typing.add(stmt).map_err(|e| typing.error_at(&e))?;
    }
    Ok(())
}
