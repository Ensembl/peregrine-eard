use std::{fmt, pin::Pin, future::Future, mem};
use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::{Return, Operation}, value::Value, interpreter::{Interpreter, InterpreterBuilder}, context::{RunContext, ContextItem}};

use super::print::{op_print, op_format};

pub trait LibcoreTemplate {
    fn print(&self, s: &str);
    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>>;
}

pub struct LibcoreBuilder {
    context: ContextItem<Box<dyn LibcoreTemplate>>
}

fn op_const(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.constants.get(regs[1]).ok_or_else(|| format!("missing constant {}",regs[1]))?;
        *ctx.registers.get_mut(regs[0])? = value.clone();
        Ok(Return::Sync)
    }))
}

fn op_copy(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone();
        ctx.set(regs[0],value)?;
        Ok(Return::Sync)
    }))
}

fn op_async(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_context = gctx.patterns.lookup::<Box<dyn LibcoreTemplate>>("libcore")?;
    Ok(Box::new(move |ctx,_regs| {
        let x = ctx.context.get(&libcore_context).call_up();
        Ok(Return::Async(x))
    }))
}

fn op_finseq_n(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteNumber(vec![]))?;
        Ok(Return::Sync)
    }))
}

fn op_infseq_n(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_number()?;
        ctx.set(regs[0],Value::InfiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

fn op_finseq_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteString(vec![]))?;
        Ok(Return::Sync)
    }))
}

fn op_infseq_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_string()?.to_string();
        ctx.set(regs[0],Value::InfiniteString(value))?;
        Ok(Return::Sync)
    }))
}

fn op_finseq_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        ctx.set(regs[0],Value::FiniteBoolean(vec![]))?;
        Ok(Return::Sync)
    }))
}

fn op_infseq_b(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.get(regs[1])?.clone().force_boolean()?;
        ctx.set(regs[0],Value::InfiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

fn op_push_n3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn op_push_n2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn op_push_s3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn op_push_s2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn op_push_b3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

fn op_push_b2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
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

pub fn build_libcore(builder: &mut InterpreterBuilder) -> Result<LibcoreBuilder,String> {
    let context = builder.add_context::<Box<dyn LibcoreTemplate>>("libcore");
    builder.add_operation(0,Operation::new(op_const));
    builder.add_operation(1,Operation::new(op_async));
    builder.add_operation(3,Operation::new(op_push_n3));
    builder.add_operation(4,Operation::new(op_push_n2));
    builder.add_operation(8,Operation::new(op_infseq_n));
    builder.add_operation(9,Operation::new(op_finseq_n));
    builder.add_operation(21,Operation::new(op_copy));
    builder.add_operation(41,Operation::new(op_infseq_s));
    builder.add_operation(42,Operation::new(op_finseq_s));
    builder.add_operation(43,Operation::new(op_push_s3));
    builder.add_operation(44,Operation::new(op_push_s2));
    builder.add_operation(51,Operation::new(op_infseq_b));
    builder.add_operation(52,Operation::new(op_finseq_b));
    builder.add_operation(53,Operation::new(op_push_b3));
    builder.add_operation(54,Operation::new(op_push_b2));
    builder.add_operation(137,Operation::new(op_print));
    builder.add_operation(138,Operation::new(op_format));
    Ok(LibcoreBuilder { context })
}

pub fn prepare_libcore<F>(context: &mut RunContext, builder: &LibcoreBuilder, libcore_template: F)
        where F: LibcoreTemplate + 'static {
    context.add(&builder.context,Box::new(libcore_template))
}
