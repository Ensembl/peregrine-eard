use std::mem;

use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return, value::Value};

pub(super) fn op_finseq_n(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteNumber(vec![]))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_infseq_n(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_number()?;
        ctx.set(regs[0],Value::InfiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_finseq_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteString(vec![]))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_infseq_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_string()?.to_string();
        ctx.set(regs[0],Value::InfiniteString(value))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_finseq_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteBoolean(vec![]))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_infseq_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_boolean()?;
        ctx.set(regs[0],Value::InfiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_n3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[1])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = ctx.force_finite_number(regs[1])?.clone();
        let value = ctx.force_number(regs[2])?;
        seq.push(value);
        ctx.set(regs[0],Value::FiniteNumber(seq))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_n2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[1])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = mem::replace(ctx.force_finite_number_mut(regs[0])?,vec![]);
        let value = ctx.force_number(regs[1])?;
        seq.push(value);
        *ctx.force_finite_number_mut(regs[0])? = seq;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_s3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[1])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = ctx.force_finite_string(regs[1])?.clone();
        let value = ctx.force_string(regs[2])?;
        seq.push(value.to_string());
        ctx.set(regs[0],Value::FiniteString(seq))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_s2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[0])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = mem::replace(ctx.force_finite_string_mut(regs[0])?,vec![]);
        let value = ctx.force_string(regs[1])?;
        seq.push(value.to_string());
        *ctx.force_finite_string_mut(regs[0])? = seq;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_b3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[1])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = ctx.force_finite_boolean(regs[1])?.clone();
        let value = ctx.force_boolean(regs[2])?;
        seq.push(value);
        ctx.set(regs[0],Value::FiniteBoolean(seq))?;
        Ok(Return::Sync)
    }))
}

pub(super) fn op_push_b2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        if !ctx.is_finite(regs[0])? {
            return Err(format!("cannot push onto infinite list"));
        }
        let mut seq = mem::replace(ctx.force_finite_boolean_mut(regs[0])?,vec![]);
        let value = ctx.force_boolean(regs[1])?;
        seq.push(value);
        *ctx.force_finite_boolean_mut(regs[0])? = seq;
        Ok(Return::Sync)
    }))
}
