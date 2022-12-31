use std::mem;

use crate::controller::{globalcontext::{GlobalContext, GlobalBuildContext}, operation::Return, value::{Value, ValueVariety}};

pub(crate) fn op_repeat(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let count = ctx.force_number(regs[2])? as usize;
        if count > 1_000_000_000 {
            return Err(format!("cowardly refusing to generate stupidly sized array"));
        }
        let value = ctx.get(regs[1])?;
        let out = match value {
            Value::Boolean(b) => Value::FiniteBoolean(vec![*b;count]),
            Value::Number(n) => Value::FiniteNumber(vec![*n;count]),
            Value::String(s) => Value::FiniteString(vec![s.to_string();count]),
            _ => { return Err(format!("cannot repeat sequence")); }
        };
        ctx.set(regs[0],out)?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_if(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let src = if ctx.force_boolean(regs[1])? { 2 } else { 3 };
        let value = ctx.get(regs[src])?;
        ctx.set(regs[0],value.clone())?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_if_seq(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let cond = mem::replace(ctx.force_finite_boolean_mut(regs[1])?,vec![]);
            let v1 = mem::replace(ctx.get_mut(regs[2])?,Value::Boolean(false));
            let v2 = mem::replace(ctx.get_mut(regs[3])?,Value::Boolean(false));
            let mut c = cond.iter();
            let cb_b = |a: &mut bool,b: &bool| {
                if !c.next().cloned().unwrap_or(true) { *a = *b; }
            };
            let mut c = cond.iter();
            let cb_n = |a: &mut f64,b: &f64| {
                if !c.next().cloned().unwrap_or(true) { *a = *b; }
            };
            let mut c = cond.iter();
            let cb_s = |a: &mut String,b: &String| {
                if !c.next().cloned().unwrap_or(true) { *a = b.clone(); }
            };
            let mut value = v1.clone();
            value.merge2(&v2,cb_b,cb_n,cb_s)?;
            ctx.set(regs[0],value)?;
            *ctx.get_mut(regs[1])? = Value::FiniteBoolean(cond);
            *ctx.get_mut(regs[2])? = v1;
            *ctx.get_mut(regs[3])? = v2;
        } else {
            let src = if ctx.force_infinite_boolean(regs[1])? { 2 } else { 3 };
            ctx.set(regs[0],ctx.get(regs[src])?.clone())?;
        }
        Ok(Return::Sync)
    }))
}
