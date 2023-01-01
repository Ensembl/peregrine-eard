use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, value::Value, operation::Return};

fn apply_template(template: &str, values: &[String]) -> Option<String> {
    let mut out = String::new();
    let mut lone_obrace = false;
    let mut lone_cbrace = false;
    let mut value = None;
    for c in template.chars() {
        if lone_obrace {
            if c == '{' { out.push('{'); }
            else if c.is_digit(10) { 
                value = c.to_string().parse::<usize>().ok();
                if value.is_none() { return None; }
            }
            else { return None; }
            lone_obrace = false;
        } else if lone_cbrace {
            if c == '}' { out.push('}'); }
            else { return None; }
            lone_cbrace = false;
        } else if value.is_some() {
            if c == '}' { out.push_str(&values[value.unwrap()]); value = None; }
            else if c.is_digit(10) {
                let more = c.to_string().parse::<usize>().ok();
                if more.is_none() { return None; }
                value = Some(value.unwrap()*10+more.unwrap());
            }
        } else if c == '{' {
            lone_obrace = true;
        } else if c == '}' {
            lone_cbrace = true;
        } else {
            out.push(c);
        }
    }
    if lone_obrace || lone_cbrace || value.is_some() { return None; }
    return Some(out);
}

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

pub(crate) fn op_split(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let sep = ctx.force_string(regs[1])?;
        let input = ctx.force_string(regs[2])?;
        let out = input.split(sep).map(|x| x.to_string()).collect();
        ctx.set(regs[0],Value::FiniteString(out))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_template(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    Ok(Box::new(move |ctx,regs| {
        let template = ctx.force_string(regs[1])?;
        let input = ctx.force_finite_string(regs[2])?;
        let out = apply_template(template,&input).unwrap_or("".to_string());
        ctx.set(regs[0],Value::String(out))?;
        Ok(Return::Sync)
    }))
}
