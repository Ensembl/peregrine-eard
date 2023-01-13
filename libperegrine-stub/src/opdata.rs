use eard_interp::{Value, GlobalBuildContext, HandleStore, GlobalContext, Return};

use crate::{data::{DataValue, Response}, stubs::{ProgramShapesBuilder, Request}};

pub(crate) fn op_request(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    Ok(Box::new(move |ctx,regs| {
        let backend = ctx.force_string(regs[1])?.to_string();
        let endpoint = ctx.force_string(regs[2])?.to_string();
        let req = Request::new(&backend,&endpoint);
        let requests = ctx.context.get_mut(&requests);
        let h = requests.push(req);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_scope(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[0])? as usize;
        let k = ctx.force_string(regs[1])?.to_string();
        let v = ctx.force_string(regs[2])?.to_string();
        let requests = ctx.context.get_mut(&requests);
        let req = requests.get_mut(h)?;
        req.scope(&k,&v);
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_get_data(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let requests = gctx.patterns.lookup::<HandleStore<Request>>("requests")?;
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    let shapes = gctx.patterns.lookup::<ProgramShapesBuilder>("shapes")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let requests = ctx.context.get(&requests);
        let req = requests.get(h)?.clone();
        let shapes = ctx.context.get_mut(&shapes);
        let res = shapes.add_request(&req)?;
        let responses = ctx.context.get_mut(&responses);
        let h = responses.push(res);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_boolean(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::Boolean(b) => b.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteBoolean(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_number(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::Number(n) => n.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteNumber(value))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_data_string(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let responses = gctx.patterns.lookup::<HandleStore<Response>>("responses")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let stream = ctx.force_string(regs[2])?;
        let responses = ctx.context.get(&responses);
        let res = responses.get(h)?.clone();
        let value = match res.get(stream)? {
            DataValue::Empty => vec![],
            DataValue::String(n) => n.to_vec(),
            _ => { return Err(format!("stream has wrong data type")) }
        };
        ctx.set(regs[0],Value::FiniteString(value))?;
        Ok(Return::Sync)
    }))
}
