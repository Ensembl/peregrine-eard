use std::fmt;
use crate::controller::{value::Value, globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return};
use super::libcore::LibcoreTemplate;

fn reduce_num(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}",n as i64)
    } else {
        format!("{}",n)
    }
}

fn fmt_seq<T: fmt::Debug>(x: &[T]) -> String {
    x.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(",")
}

fn fmt_seq_num(x: &[f64]) -> String {
    x.iter().map(|x| reduce_num(*x)).collect::<Vec<_>>().join(",")
}

pub(crate) fn op_format(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(|ctx,regs| {
        let value = ctx.registers.get(regs[1])?;
        let out = match value {
            Value::Boolean(b) => format!("{:?}",b),
            Value::Number(n) => reduce_num(*n),
            Value::String(s) => format!("{}",s),
            Value::FiniteBoolean(b) => format!("[{}]",fmt_seq(b)),
            Value::FiniteNumber(n) => format!("[{}]",fmt_seq_num(n)),
            Value::FiniteString(s) => format!("[{}]",fmt_seq(s)),
            Value::InfiniteBoolean(b) => format!("[{:?},...]",b),
            Value::InfiniteNumber(n) => format!("[{},...]",n),
            Value::InfiniteString(s) => format!("[{:?},...]",s),
        };
        *ctx.registers.get_mut(regs[0]).expect("bad reg") = Value::String(out);
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_print(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let libcore_context = gctx.patterns.lookup::<Box<dyn LibcoreTemplate>>("libcore")?;
    Ok(Box::new(move |ctx,regs| {
        let value = ctx.force_string(regs[0])?;
        ctx.context.get(&libcore_context).print(value);
        Ok(Return::Sync)
    }))
}
