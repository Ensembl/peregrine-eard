use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, value::Value, operation::Return};

pub(super) fn op_len_n(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = if !ctx.is_finite(regs[1])? {
            -1
        } else {
            let seq = ctx.force_finite_number(regs[1])?.clone();
            seq.len() as i32
        };
        ctx.set(regs[0],Value::Number(value as f64))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_len_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = if !ctx.is_finite(regs[1])? {
            -1
        } else {
            let seq = ctx.force_finite_string(regs[1])?.clone();
            seq.len() as i32
        };
        ctx.set(regs[0],Value::Number(value as f64))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_len_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = if !ctx.is_finite(regs[1])? {
            -1
        } else {
            let seq = ctx.force_finite_boolean(regs[1])?.clone();
            seq.len() as i32
        };
        ctx.set(regs[0],Value::Number(value as f64))?;
        Ok(Return::Sync)
    }))
}

fn build_check<F>(ctx: &mut GlobalContext, reg: usize, cb: F) -> Result<Option<usize>,String>
        where F: Fn(&mut usize,usize) {
    Ok(if !ctx.is_finite(reg)? {
        None
    } else {
        let values = ctx.force_finite_number(reg)?;
        let mut bound = 0;
        for value in values.iter() {
            if value.fract() != 0. && *value > 0. {
                return Ok(None);
            }
            cb(&mut bound,*value as usize);
        }
        Some(bound)
    })
}

pub(super) fn op_total(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if let Some(total) = build_check(ctx,regs[1],|b,v| *b += v)? {
            ctx.set(regs[0],Value::Number(total as f64))?;
            Ok(Return::Sync)
        } else {
            Err(format!("failed total check"))
        }
    }))
}

pub(super) fn op_bound(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if let Some(total) = build_check(ctx,regs[1],|b,v| *b = (*b).max(v))? {
            ctx.set(regs[0],Value::Number(total as f64))?;
            Ok(Return::Sync)
        } else {
            Err(format!("failed ref check"))
        }
    }))
}

fn check_or_fail<F>(check_name: &str, cb: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(i32,i32) -> bool + 'static {
    let check_name = check_name.to_string();
    Ok(Box::new(move |ctx,regs| {
        let msg = ctx.force_string(regs[0])?;
        let a = ctx.force_number(regs[1])? as i32;
        let b = ctx.force_number(regs[2])? as i32;
        if !cb(a,b) {
            return Err(msg.to_string());
        }
        Ok(Return::Sync)
    }))
}

pub(super) fn op_check_l(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("length",|a,b| a==b)
}

pub(super) fn op_check_tt(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("length/total",|a,b| a==b)
}

pub(super) fn op_check_t(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("total",|a,b| a==b)
}

pub(super) fn op_check_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("ref",|a,b| a>b)
}

pub(super) fn op_check_li(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("inf/inf",|a,b| a==b || b==-1)
}

pub(super) fn op_check_ii(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    check_or_fail("inf/inf",|a,b| a==b || a==-1 || b==-1)
}
