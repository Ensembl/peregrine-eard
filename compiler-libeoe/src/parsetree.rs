use std::collections::HashMap;

use eard_compiler_lib::{PTExpression, Constant, Variable};
use ordered_float::OrderedFloat;

use crate::structmacro::StructMacro;

#[derive(Clone,Debug)]
pub(crate) enum VarType { Number, String, Boolean }

impl VarType {
    pub(crate) fn to_var_call(&self) -> String {
        match self {
            VarType::Boolean => "eoe_var_boolean",
            VarType::Number => "eoe_var_number",
            VarType::String => "eoe_var_string",
        }.to_string()
    }
}

#[derive(Debug)]
pub(crate) enum StructExpr {
    Object(Vec<(String,StructExpr)>),
    Array(Vec<StructExpr>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Var(Option<usize>,VarType,usize),
    Cond(usize,Box<StructExpr>),
    All(usize,Vec<usize>,Box<StructExpr>)
}

impl StructExpr {
    pub(crate) fn visit<F>(&mut self, cb: &mut F) -> Result<(),String> where F: FnMut(&mut StructExpr) -> Result<(),String> {
        cb(self)?;
        match self {
            StructExpr::Object(j) => {
                for (_,x) in j {
                    x.visit(cb)?;
                }
            },
            StructExpr::Array(a) => {
                for x in a {
                    x.visit(cb)?;
                }
            },
            StructExpr::Cond(_,x) => {
                x.visit(cb)?;
            },
            StructExpr::All(_,_,x) => {
                x.visit(cb)?;
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn arg_types(&mut self, out: &mut Vec<VarType>) {
        self.visit(&mut |x| {
            match x {
                StructExpr::Var(_,t,i) => {
                    if out.len() <= *i {
                        out.resize_with(*i+1,|| VarType::Boolean);
                    }
                    out[*i] = t.clone();
                },
                StructExpr::Cond(i,_) => {
                    if out.len() <= *i {
                        out.resize_with(*i+1,|| VarType::Boolean);
                    }
                },
                _ => {}
            }
            Ok(())
        }).ok().unwrap();
    }

    pub(crate) fn assign_all_ids(&mut self, next: &mut usize, var_groups: &mut HashMap<usize,usize>) -> Result<(),String> {
        self.visit(&mut |x| {
            match x {
                StructExpr::All(id,vars,_) => {
                    *id = *next;
                    for var in vars {
                        if var_groups.contains_key(var) {
                            return Err(format!(""));
                        }
                        var_groups.insert(*var,*id);
                    }
                    *next += 1;
                },
                _ => {}
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn build_expression(&self, sm: &StructMacro) -> Result<PTExpression,String> {
        Ok(match self {
            StructExpr::Object(j) => {
                let exprs = j.iter().map(|(k,v)| {
                    let pair = sm.call("eoe_pair",&[
                        PTExpression::Constant(Constant::String(k.to_string())),
                        v.build_expression(sm)?
                    ])?;
                    Ok::<_,String>(pair)
                }).collect::<Result<Vec<_>,_>>()?;
                sm.call("eoe_object",&[PTExpression::FiniteSequence(exprs)])?
            },
            StructExpr::Array(a) => {
                let exprs = a.iter().map(|x| 
                    Ok::<_,String>(x.build_expression(sm)?)
                ).collect::<Result<Vec<_>,_>>()?;
                sm.call("eoe_array",&[PTExpression::FiniteSequence(exprs)])?
            },
            StructExpr::String(s) => {
                sm.call("eoe_string",&[PTExpression::Constant(Constant::String(s.to_string()))])?
            },
            StructExpr::Number(n) => {
                sm.call("eoe_number",&[PTExpression::Constant(Constant::Number(OrderedFloat(*n)))])?
            },
            StructExpr::Boolean(b) => {
                sm.call("eoe_boolean",&[PTExpression::Constant(Constant::Boolean(*b))])?
            },
            StructExpr::Null => {
                sm.call("eoe_null",&[])?
            },
            StructExpr::Var(_,_,i) => {
                let name = sm.var_vars[*i].clone().ok_or_else(||
                    format!("variable was not iterated over")
                )?;
                sm.call("eoe_var",&[
                    PTExpression::Variable(Variable { prefix: None, name })
                ])?
            },
            StructExpr::Cond(i,x) => {
                let name = sm.var_vars[*i].clone().ok_or_else(||
                    format!("cond variable was not iterated over")
                )?;
                sm.call("eoe_condition",&[
                    PTExpression::Variable(Variable { prefix: None, name }),
                    x.build_expression(sm)?
                ])?
            },
            StructExpr::All(i,_,x) => {
                sm.call("eoe_all",&[
                    PTExpression::Variable(Variable { prefix: None, name: sm.group_vars[*i].clone() }),
                    x.build_expression(sm)?
                ])?
            },
        })
    }
}
