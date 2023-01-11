use eachorevery::{eoestruct::{StructTemplate, StructVarGroup, StructVar, StructPair}, EachOrEvery};
use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Return, Value};

pub(crate) fn op_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let input = ctx.force_boolean(regs[1])?;
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_boolean(input));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let input = ctx.force_number(regs[1])?;
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_number(input));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let input = ctx.force_string(regs[1])?.to_string();
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_string(input));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_null(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_null());
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_group(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let groups = gctx.patterns.lookup::<HandleStore<StructVarGroup>>("eoegroups")?;
    Ok(Box::new(move |ctx,regs| {
        let groups = ctx.context.get_mut(&groups);
        let h = groups.push(StructVarGroup::new());
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn eoe_boolean(ctx: &GlobalContext, reg: usize) -> Result<EachOrEvery<bool>,String> {
    Ok(if ctx.is_finite(reg)? {
        EachOrEvery::each(ctx.force_finite_boolean(reg)?.to_vec())
    } else {
        EachOrEvery::every(ctx.force_infinite_boolean(reg)?)
    })
}

pub(crate) fn op_var_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let groups = gctx.patterns.lookup::<HandleStore<StructVarGroup>>("eoegroups")?;
    let vars = gctx.patterns.lookup::<HandleStore<StructVar>>("eoevars")?;
    Ok(Box::new(move |ctx,regs| {
        let group_h = ctx.force_number(regs[1])? as usize;
        let input = eoe_boolean(ctx,regs[2])?;
        let groups = ctx.context.get_mut(&groups);
        let mut group = groups.get_mut(group_h)?;
        let value = StructVar::new_boolean(&mut group,input);
        let vars = ctx.context.get_mut(&vars);
        let h = vars.push(value);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn eoe_number(ctx: &GlobalContext, reg: usize) -> Result<EachOrEvery<f64>,String> {
    Ok(if ctx.is_finite(reg)? {
        EachOrEvery::each(ctx.force_finite_number(reg)?.to_vec())
    } else {
        EachOrEvery::every(ctx.force_infinite_number(reg)?)
    })
}

pub(crate) fn op_var_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let groups = gctx.patterns.lookup::<HandleStore<StructVarGroup>>("eoegroups")?;
    let vars = gctx.patterns.lookup::<HandleStore<StructVar>>("eoevars")?;
    Ok(Box::new(move |ctx,regs| {
        let group_h = ctx.force_number(regs[1])? as usize;
        let input = eoe_number(ctx,regs[2])?;
        let groups = ctx.context.get_mut(&groups);
        let mut group = groups.get_mut(group_h)?;
        let value = StructVar::new_number(&mut group,input);
        let vars = ctx.context.get_mut(&vars);
        let h = vars.push(value);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn eoe_string(ctx: &GlobalContext, reg: usize) -> Result<EachOrEvery<String>,String> {
    Ok(if ctx.is_finite(reg)? {
        EachOrEvery::each(ctx.force_finite_string(reg)?.to_vec())
    } else {
        EachOrEvery::every(ctx.force_infinite_string(reg)?.to_string())
    })
}

pub(crate) fn op_var_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let groups = gctx.patterns.lookup::<HandleStore<StructVarGroup>>("eoegroups")?;
    let vars = gctx.patterns.lookup::<HandleStore<StructVar>>("eoevars")?;
    Ok(Box::new(move |ctx,regs| {
        let group_h = ctx.force_number(regs[1])? as usize;
        let input = eoe_string(ctx,regs[2])?;
        let groups = ctx.context.get_mut(&groups);
        let mut group = groups.get_mut(group_h)?;
        let value = StructVar::new_string(&mut group,input);
        let vars = ctx.context.get_mut(&vars);
        let h = vars.push(value);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_array(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let input_h = ctx.force_finite_number(regs[1])?.to_vec();
        let templates = ctx.context.get_mut(&templates);
        let input = input_h.iter().map(|h| templates.get(*h as usize).cloned()).collect::<Result<Vec<_>,_>>()?;
        let h = templates.push(StructTemplate::new_array(input));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_pair(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let pairs = gctx.patterns.lookup::<HandleStore<StructPair>>("eoepairs")?;
    Ok(Box::new(move |ctx,regs| {
        let key = ctx.force_string(regs[1])?.to_string();
        let value_h = ctx.force_number(regs[2])?;
        let templates = ctx.context.get(&templates);
        let value = templates.get(value_h as usize)?.clone();
        let pairs = ctx.context.get_mut(&pairs);
        let h = pairs.push(StructPair::new(&key,value));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_object(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let pairs = gctx.patterns.lookup::<HandleStore<StructPair>>("eoepairs")?;
    Ok(Box::new(move |ctx,regs| {
        let input_h = ctx.force_finite_number(regs[1])?.to_vec();
        let pairs = ctx.context.get_mut(&pairs);
        let input = input_h.iter().map(|h| pairs.get(*h as usize).cloned()).collect::<Result<Vec<_>,_>>()?;
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_object(input));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_var(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let vars = gctx.patterns.lookup::<HandleStore<StructVar>>("eoevars")?;
    Ok(Box::new(move |ctx,regs| {
        let value_h = ctx.force_number(regs[1])? as usize;
        let vars = ctx.context.get(&vars);
        let value = vars.get(value_h as usize)?.clone();
        let templates = ctx.context.get_mut(&templates);
        let h = templates.push(StructTemplate::new_var(&value));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_all(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let groups = gctx.patterns.lookup::<HandleStore<StructVarGroup>>("eoegroups")?;
    Ok(Box::new(move |ctx,regs| {
        let group_h = ctx.force_number(regs[1])? as usize;
        let templates_r = ctx.context.get(&templates);
        let inner = templates_r.get(ctx.force_number(regs[2])? as usize)?.clone();
        let groups = ctx.context.get_mut(&groups);
        let group = groups.get_mut(group_h)?;
        let all = StructTemplate::new_all(group,inner);
        let templates_w = ctx.context.get_mut(&templates);
        let h = templates_w.push(all);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_condition(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let vars = gctx.patterns.lookup::<HandleStore<StructVar>>("eoevars")?;
    Ok(Box::new(move |ctx,regs| {
        let templates_r = ctx.context.get(&templates);
        let inner = templates_r.get(ctx.force_number(regs[2])? as usize)?.clone();
        let vars = ctx.context.get(&vars);
        let var = vars.get(ctx.force_number(regs[1])? as usize)?;
        let all = StructTemplate::new_condition(var.clone(),inner);
        let templates_w = ctx.context.get_mut(&templates);
        let h = templates_w.push(all);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}
