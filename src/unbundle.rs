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

    fn unbundle_callarg(&self, x: &CallArg<BTExpression>) -> Result<Option<String>,String> {
        Ok(match x {
            CallArg::Expression(BTExpression::Function(f)) => {
                let defn = match &self.bt.definitions[f.func_index] {
                    BTDefinition::Func(f) => f,
                    _ => { panic!("expected func definition"); }
                };
                self.unbundle_expression(&defn.ret[0])?
            },
            CallArg::Expression(_) => None,
            CallArg::Bundle(b) => Some(b.to_string()),
            CallArg::Repeater(_) => { panic!("unexpected repeater"); }
        })
    }

    fn unbundle_expression(&self, x: &OrBundle<BTExpression>) -> Result<Option<String>,String> {
        Ok(match x {
            OrBundle::Normal(BTExpression::Function(f)) => {
                self.unbundle_expression(&self.bt.get_function(f)?.ret[0])?
            },
            OrBundle::Normal(_) => None,
            OrBundle::Bundle(b) => Some(b.to_string())
        })
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

    fn unbundle_defn(&mut self, defn: &BTFuncProcDefinition, ret_outsides: &[Option<String>], arg_outsides: &[Option<String>]) -> Result<(),String> {
        /* return expression */
        let ret_insides = defn.ret.iter().map(|x| {
            self.unbundle_expression(x)
        }).collect::<Result<Vec<_>,_>>()?;
        self.bundle_match(&ret_insides,ret_outsides,true)?;
        /* statements */
        for stmt in defn.block.iter().rev() {
            self.unbundle_stmt(stmt)?;
        }
        /* arguments */
        let arg_insides = defn.args.iter().map(|x| {
            match x {
                OrBundle::Normal(_) => None,
                OrBundle::Bundle(b) => Some(b.to_string())
            }
        }).collect::<Vec<_>>();
        self.bundle_match(&arg_insides,arg_outsides,false)?;
        Ok(())
    }

    fn bundle_match(&self, insides: &[Option<String>], outsides: &[Option<String>], ret: bool) -> Result<(),String> {
        for (inside,outside) in insides.iter().zip(outsides.iter()) {
            match (inside,outside) {
                (Some(inside),Some(outside)) => {
                    if ret {
                        println!("> bundle return match inside {:?} outside {:?}",inside,outside);
                    } else {
                        println!("> bundle arg match inside {:?} outside {:?}",inside,outside);
                    }
                },
                (None,None) => {},
                _ => {
                    return Err(self.error_at("unexpected bundle return"));
                }
            }
        }
        Ok(())
    }

    fn unbundle_proccall(&mut self, call: &BTProcCall) -> Result<(),String> {
        if self.uses_repeater(call)? {
            if self.is_good_repeater(call)? {
                self.unbundle_repeater(call)?;
            } else {
                return Err(self.error_at("bad repeater statement"));
            }
        } else {
            let ret_outsides = call.rets.as_ref().unwrap_or(&vec![]).iter().map(|ret| {
                match ret {
                    BTLValue::Bundle(b) => Some(b.to_string()),
                    _ => None
                }
            }).collect::<Vec<_>>();
            let arg_outsides = call.args.iter().map(|arg| {
                match arg {
                    CallArg::Expression(_) => None,
                    CallArg::Bundle(b) => Some(b.to_string()),
                    CallArg::Repeater(_) => { panic!("Unexpected repeater"); }
                }
            }).collect::<Vec<_>>();
            if let Some(defn) = self.bt.get_procedure(call)? {
                self.unbundle_defn(defn,&ret_outsides,&arg_outsides)?;
            } else {
                let inside = call.args.iter().map(|x| {
                    self.unbundle_callarg(x)
                }).collect::<Result<Vec<_>,_>>()?;
                self.bundle_match(&inside,&ret_outsides,true)?   
            }
        }
        Ok(())
    }

    fn unbundle_stmt(&mut self, stmt: &BTStatement) -> Result<(),String> {
        println!("B {:?}",stmt);
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            BTStatementValue::Declare(d) => self.unbundle_declare(d)?,
            BTStatementValue::Statement(s) => self.unbundle_proccall(s)?,
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn unbundle(&mut self) -> Result<(),String> {
        println!("A");
        for stmt in self.bt.statements.iter().rev() {
            self.unbundle_stmt(stmt)?;
        }
        Ok(())
    }
}
