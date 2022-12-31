use crate::controller::{globalcontext::{GlobalContext, GlobalBuildContext}, operation::Return, value::Value};

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
