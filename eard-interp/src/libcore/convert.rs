use crate::controller::{value::Value, globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return};

pub(crate) fn op_to_bool_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.,
            Value::String(s) => s != "",
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::Boolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_bool(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.,
            Value::String(s) => s != "",
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::Boolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_bool_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::FiniteBoolean(b) => Value::FiniteBoolean(b.clone()),
            Value::FiniteNumber(n) => Value::FiniteBoolean(n.iter().map(|v| *v != 0.).collect()),
            Value::FiniteString(s) => Value::FiniteBoolean(s.iter().map(|v| v != "").collect()),
            Value::InfiniteBoolean(b) => Value::InfiniteBoolean(*b),
            Value::InfiniteNumber(n) => Value::InfiniteBoolean(*n != 0.),
            Value::InfiniteString(s) => Value::InfiniteBoolean(s != ""),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],value)?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_bool_s_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::FiniteBoolean(b) => None,
            Value::FiniteNumber(n) => Some(Value::FiniteBoolean(n.iter().map(|v| *v != 0.).collect())),
            Value::FiniteString(s) => Some(Value::FiniteBoolean(s.iter().map(|v| v != "").collect())),
            Value::InfiniteBoolean(b) => None,
            Value::InfiniteNumber(n) => Some(Value::InfiniteBoolean(*n != 0.)),
            Value::InfiniteString(s) => Some(Value::InfiniteBoolean(s != "")),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        if let Some(value) = value {
            ctx.set(regs[0],value)?;
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_num_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::Boolean(b) => if *b { 1. } else { 0. },
            Value::Number(n) => *n,
            Value::String(s) => s.parse::<f64>().unwrap_or(0.),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::Number(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_num(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::Boolean(b) => if *b { 1. } else { 0. },
            Value::Number(n) => *n,
            Value::String(s) => s.parse::<f64>().unwrap_or(0.),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::Number(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_num_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::FiniteBoolean(b) => Value::FiniteNumber(b.iter().map(|v| if *v { 1. } else { 0. }).collect()),
            Value::FiniteNumber(n) => Value::FiniteNumber(n.clone()),
            Value::FiniteString(s) => Value::FiniteNumber(s.iter().map(|v| v.parse::<f64>().unwrap_or(0.)).collect()),
            Value::InfiniteBoolean(b) => Value::InfiniteNumber(if *b { 1. } else { 0. }),
            Value::InfiniteNumber(n) => Value::InfiniteNumber(*n),
            Value::InfiniteString(s) => Value::InfiniteNumber(s.parse::<f64>().unwrap_or(0.)),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],value)?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_num_s_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::FiniteBoolean(b) => Some(Value::FiniteNumber(b.iter().map(|v| if *v { 1. } else { 0. }).collect())),
            Value::FiniteNumber(n) => None,
            Value::FiniteString(s) => Some(Value::FiniteNumber(s.iter().map(|v| v.parse::<f64>().unwrap_or(0.)).collect())),
            Value::InfiniteBoolean(b) => Some(Value::InfiniteNumber(if *b { 1. } else { 0. })),
            Value::InfiniteNumber(n) => None,
            Value::InfiniteString(s) => Some(Value::InfiniteNumber(s.parse::<f64>().unwrap_or(0.))),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        if let Some(value) = value {
            ctx.set(regs[0],value)?;
        }
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_str_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_string(),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::String(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_str(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_string(),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],Value::String(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_str_s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[1])? {
            Value::FiniteBoolean(b) => Value::FiniteString(b.iter().map(|v| if *v { "true".to_string() } else { "false".to_string() }).collect()),
            Value::FiniteNumber(n) => Value::FiniteString(n.iter().map(|v| v.to_string()).collect()),
            Value::FiniteString(s) => Value::FiniteString(s.clone()),
            Value::InfiniteBoolean(b) => Value::InfiniteString(if *b { "true".to_string() } else { "false".to_string() }),
            Value::InfiniteNumber(n) => Value::InfiniteString(n.to_string()),
            Value::InfiniteString(s) => Value::InfiniteString(s.to_string()),
            _ => { return Err(format!("bad type converting to bool")); }
        };
        ctx.set(regs[0],value)?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_to_str_s_m(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let value = match ctx.get(regs[0])? {
            Value::FiniteBoolean(b) => Some(Value::FiniteString(b.iter().map(|v| if *v { "true".to_string() } else { "false".to_string() }).collect())),
            Value::FiniteNumber(n) => Some(Value::FiniteString(n.iter().map(|v| v.to_string()).collect())),
            Value::FiniteString(s) => None,
            Value::InfiniteBoolean(b) => Some(Value::InfiniteString(if *b { "true".to_string() } else { "false".to_string() })),
            Value::InfiniteNumber(n) => Some(Value::InfiniteString(n.to_string())),
            Value::InfiniteString(s) => None,
            _ => { return Err(format!("bad type converting to bool")); }
        };
        if let Some(value) = value {
            ctx.set(regs[0],value)?;
        }
        Ok(Return::Sync)
    }))
}
