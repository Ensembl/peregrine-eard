use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, value::Value, operation::Return};

pub(crate) fn op_concat(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let sep = ctx.force_string(regs[1])?;
        let parts = ctx.force_finite_string(regs[2])?;
        ctx.set(regs[0],Value::String(parts.join(sep)))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let first = ctx.force_string(regs[1])?;
        let second = ctx.force_string(regs[2])?;
        ctx.set(regs[0],Value::String(format!("{}{}",first,second)))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let base = ctx.force_finite_string(regs[1])?;
            let extra = ctx.force_string(regs[2])?;
            let out = base.iter().map(|s| format!("{}{}",s,extra)).collect();
            ctx.set(regs[0],Value::FiniteString(out))?;    
        } else {
            let first = ctx.force_string(regs[1])?;
            let second = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::InfiniteString(format!("{}{}",first,second)))?;    
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_push_str_revs(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let extra = ctx.force_string(regs[1])?;
            let base = ctx.force_finite_string(regs[2])?;
            let out = base.iter().map(|s| format!("{}{}",extra,s)).collect();
            ctx.set(regs[0],Value::FiniteString(out))?;    
        } else {
            let first = ctx.force_string(regs[1])?;
            let second = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::InfiniteString(format!("{}{}",first,second)))?;    
        }
        Ok(Return::Sync)
    }))
}
