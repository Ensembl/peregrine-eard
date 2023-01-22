use eard_interp::{GlobalBuildContext, GlobalContext, Return, Value };
use crate::{stubs::ProgramShapesBuilder, util::{to_string, to_number, to_boolean}};

pub(crate) fn op_setting_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_boolean(&shapes.get_setting(key,path)?);
        let value = value.iter().any(|v| *v);
        ctx.set(regs[0],Value::Boolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let mut value = to_number(&shapes.get_setting(key,path)?);
        let value = value.drain(..).reduce(f64::max).unwrap_or(0.);
        ctx.set(regs[0],Value::Number(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_string(&shapes.get_setting(key,path)?);
        let value = value.join("\n");
        ctx.set(regs[0],Value::String(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_boolean_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_boolean(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_number(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string_seq(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let path = ctx.force_finite_string(regs[2])?;
        let value = to_string(&shapes.get_setting(key,path)?);
        ctx.set(regs[0],Value::FiniteString(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_boolean_keys(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let mut path = ctx.force_finite_string(regs[2])?.to_vec();
        path.insert(0,"__keys".to_string());
        let value = to_boolean(&shapes.get_setting(key,&path)?);
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_number_keys(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let mut path = ctx.force_finite_string(regs[2])?.to_vec();
        path.insert(0,"__keys".to_string());
        let value = to_number(&shapes.get_setting(key,&path)?);
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_setting_string_keys(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let key = ctx.force_string(regs[1])?;
        let mut path = ctx.force_finite_string(regs[2])?.to_vec();
        path.insert(0,"__keys".to_string());
        let value = to_string(&shapes.get_setting(key,&path)?);
        ctx.set(regs[0],Value::FiniteString(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_small_value(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let shapes = ctx.context.get(&shapes);
        let namespace = ctx.force_string(regs[1])?;
        let column = ctx.force_string(regs[2])?;
        let keys = ctx.force_finite_string(regs[3])?.to_vec();
        let values = keys.iter().map(|key| {
            shapes.get_small_value(namespace,column,key)
        }).collect::<Result<Vec<_>,_>>()?;
        ctx.set(regs[0],Value::FiniteString(values))?;
        Ok(Return::Sync)
    }))
}
