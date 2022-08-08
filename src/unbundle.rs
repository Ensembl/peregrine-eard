use std::sync::Arc;

use crate::{buildtree::{BTStatement, BuildTree, BTStatementValue, BTDeclare, BTProcCall, BTLValue, BTExpression, BTFuncProcDefinition, BTDefinition}, parsetree::at, model::{CallArg, OrBundle}};

pub(crate) struct Unbundle<'a> {
    bt: &'a BuildTree,
    position: Option<(Arc<Vec<String>>,usize)>
}

impl<'a> Unbundle<'a> {
    pub(crate) fn new(bt: &'a BuildTree) -> Unbundle<'a> {
        Unbundle {
            bt,
            position: None
        }
    }

    fn error_at(&self, msg: &str) -> String {
        let pos = self.position.as_ref().map(|(f,p)| (f.as_slice(),*p));
        at(msg,pos)
    }

    fn list_repeaters(&self, args: &[CallArg<BTExpression>]) -> Result<Vec<String>,String> {
        let mut repeaters = vec![];
        for arg in args {
            match arg {
                CallArg::Expression(x) => {
                    match x {
                        BTExpression::Function(f) => {
                            repeaters.append(&mut self.list_repeaters(&f.args)?);
                        },
                        _ => {},
                    }
                },
                CallArg::Bundle(_) => {},
                CallArg::Repeater(r) => { repeaters.push(r.to_string()) }
            }
        }
        Ok(repeaters)
    }

    fn is_good_repeater(&self, call: &BTProcCall) -> Result<bool,String> {
        /* A good repeater has a single use of the repeater glyph (**) on each side and has a
         * single lhs variable.
         */
        let rets = match &call.rets {
            Some(x) if x.len() == 1 => { &x[0] },
            _ => { return Ok(false); } // len != 1 on lhs
        };
        match rets {
            BTLValue::Repeater(_) => {},
            _ => { return Ok(false); } // lhs is not repeater
        }
        Ok(self.list_repeaters(&call.args)?.len() == 1)
    }

    fn uses_repeater(&self, call: &BTProcCall) -> Result<bool,String> {
        for ret in call.rets.as_ref().unwrap_or(&vec![]).iter() {
            match ret {
                BTLValue::Repeater(_) => { return Ok(true); },
                _ => {}
            }
        }
        Ok(self.list_repeaters(&call.args)?.len() > 0)
    }

    fn found_bundle_source(&self, name: &str) {
        println!("> found bundle source for {:?}",name);
    }

    fn found_bundle_member_source(&self, name: &str, var: &str) {
        println!("> found bundle member source for {:?}.{:?}",name,var);
    }

    fn unbundle_declare(&self, decl: &BTDeclare) -> Result<(),String> {
        match decl {
            BTDeclare::Variable(v) => {
                if let Some(prefix) = &v.prefix {
                    self.found_bundle_member_source(prefix,&v.name);
                }
            },
            BTDeclare::Repeater(r) => { self.found_bundle_source(r); }
            BTDeclare::Bundle(r) => { self.found_bundle_source(r); }
        }
        Ok(())
    }

    fn unbundle_expression(&self) -> Result<OrBundle<()>,String> {
        todo!();
    }

    fn unbundle_repeater(&self, call: &BTProcCall) -> Result<(),String> {
        let source = match &call.rets.as_ref().expect("unbundling repeater from non-repeater")[0] {
            BTLValue::Repeater(r) => r,
            _ => { panic!("unbundling repeater from non-repeater {:?}",call); }
        };
        let target = self.list_repeaters(&call.args)?[0].to_string();
        println!("> repeater copy usage of {:?} into {:?}",source,target);
        Ok(())
    }

    fn unbundle_defn(&self, defn: &BTFuncProcDefinition, outside: &[Option<String>]) -> Result<(),String> {
        let inside = defn.ret.iter().map(|x| {
            match x {
                OrBundle::Normal(_) => None,
                OrBundle::Bundle(b) => Some(b.to_string())
            }
        }).collect::<Vec<_>>();
        self.bundle_return_match(&inside,outside)?;
        self.unbundle_expression();
        //todo!(); // final expression
        for stmt in defn.block.iter().rev() {
            //todo!(); // each statement
        }
        Ok(())
    }

    fn bundle_return_match(&self, insides: &[Option<String>], outsides: &[Option<String>]) -> Result<(),String> {
        for (inside,outside) in insides.iter().zip(outsides.iter()) {
            match (inside,outside) {
                (Some(inside),Some(outside)) => {
                    println!("> bundle return match inside {:?} outside {:?}",inside,outside);
                },
                (None,None) => {},
                _ => {
                    return Err(self.error_at("unexpected bundle return"));
                }
            }
        }
        Ok(())
    }

    fn unbundle_proccall(&self, call: &BTProcCall) -> Result<(),String> {
        if self.uses_repeater(call)? {
            if self.is_good_repeater(call)? {
                self.unbundle_repeater(call)?;
            } else {
                return Err(self.error_at("bad repeater statement"));
            }
        } else {
            println!("NORMAL");
            let outside = call.rets.as_ref().unwrap_or(&vec![]).iter().map(|ret| {
                match ret {
                    BTLValue::Bundle(b) => Some(b.to_string()),
                    _ => None
                }
            }).collect::<Vec<_>>();
            if let Some(index) = call.proc_index {
                let defn = match &self.bt.definitions[index] {
                    BTDefinition::Proc(p) => p,
                    _ => { panic!("expected proc definition"); }
                };
                self.unbundle_defn(defn,&outside)?;
            } else {
                let inside = call.args.iter().map(|x| {
                    match x {
                        CallArg::Bundle(b) => Some(b.to_string()),
                        _ => None
                    }
                }).collect::<Vec<_>>();
                self.bundle_return_match(&inside,&outside)?;
            }
        }
        Ok(())
    }

    pub(crate) fn unbundle(&mut self) -> Result<(),String> {
        println!("A");
        for stmt in self.bt.statements.iter().rev() {
            println!("B {:?}",stmt);
            self.position = Some((stmt.file.clone(),stmt.line_no));
            match &stmt.value {
                BTStatementValue::Declare(d) => self.unbundle_declare(d)?,
                BTStatementValue::Statement(s) => self.unbundle_proccall(s)?,
                _ => {}
            }
        }
        Ok(())
    }
}
