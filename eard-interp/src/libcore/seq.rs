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

fn set_merge(ctx: &GlobalContext, dst: &mut Value, cond_reg: usize, repl: &Value, advance: bool) -> Result<(),String> {
    if ctx.is_finite(cond_reg)? {
        let cond = ctx.force_finite_boolean(cond_reg)?;
        let mut c = cond.iter();
        let cb_b = |a: &mut bool,b: &bool| {
            if c.next().cloned().unwrap_or(true) { *a = *b; true } else { advance }
        };
        let mut c = cond.iter();
        let cb_n = |a: &mut f64,b: &f64| {
            if c.next().cloned().unwrap_or(true) { *a = *b; true } else { advance }
        };
        let mut c = cond.iter();
        let cb_s = |a: &mut String,b: &String| {
            if c.next().cloned().unwrap_or(true) { *a = b.clone(); true } else { advance }
        };
        dst.merge2(&repl,cb_b,cb_n,cb_s)
    } else {
        if ctx.force_infinite_boolean(cond_reg)? {
            *dst = repl.clone();
        }
        Ok(())
    }
}

pub(crate) fn op_set(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[2])? {
            let src = ctx.get(regs[1])?;
            let mut dst = src.clone();
            set_merge(ctx,&mut dst,regs[2],ctx.get(regs[3])?,false)?;
            ctx.set(regs[0],dst)?;
        } else {
            let src = if ctx.force_infinite_boolean(regs[2])? { 3 } else { 1 };
            ctx.set(regs[0],ctx.get(regs[src])?.clone())?;
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_set_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let mut value = mem::replace(ctx.get_mut(regs[0])?,Value::Boolean(false));
            set_merge(ctx,&mut value,regs[1],ctx.get(regs[2])?,false)?;
            ctx.set(regs[0],value)?;
        } else {
            if ctx.force_infinite_boolean(regs[1])? {
                ctx.set(regs[0],ctx.get(regs[2])?.clone())?;
            }
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_set_skip(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[2])? {
            let src = ctx.get(regs[1])?;
            let mut dst = src.clone();
            set_merge(ctx,&mut dst,regs[2],ctx.get(regs[3])?,true)?;
            ctx.set(regs[0],dst)?;
        } else {
            let src = if ctx.force_infinite_boolean(regs[2])? { 3 } else { 1 };
            ctx.set(regs[0],ctx.get(regs[src])?.clone())?;
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_set_skip_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let mut value = mem::replace(ctx.get_mut(regs[0])?,Value::Boolean(false));
            set_merge(ctx,&mut value,regs[1],ctx.get(regs[2])?,true)?;
            ctx.set(regs[0],value)?;
        } else {
            if ctx.force_infinite_boolean(regs[1])? {
                ctx.set(regs[0],ctx.get(regs[2])?.clone())?;
            }
        }
        Ok(Return::Sync)
    }))
}
