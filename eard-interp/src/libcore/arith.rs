use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return, value::Value};

fn op_num2<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        let a = ctx.force_number(regs[0])?;
        let b = ctx.force_number(regs[1])?;
        ctx.set(regs[0],Value::Number(f(a,b)))?;
        Ok(Return::Sync)
    }))
}

fn op_num3<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        let a = ctx.force_number(regs[1])?;
        let b = ctx.force_number(regs[2])?;
        ctx.set(regs[0],Value::Number(f(a,b)))?;
        Ok(Return::Sync)
    }))
}

fn op_num2s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let a = ctx.force_finite_number(regs[0])?;
            let b = ctx.force_number(regs[1])?;
            let v = a.iter().map(|a| f(*a,b)).collect();
            ctx.set(regs[0],Value::FiniteNumber(v))?;
        } else {
            let a = ctx.force_infinite_number(regs[0])?;
            let b = ctx.force_number(regs[1])?;
            ctx.set(regs[0],Value::InfiniteNumber(f(a,b)))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_num3s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let a = ctx.force_finite_number(regs[1])?;
            let b = ctx.force_number(regs[2])?;
            let v = a.iter().map(|a| f(*a,b)).collect();
            ctx.set(regs[0],Value::FiniteNumber(v))?;
        } else {
            let a = ctx.force_infinite_number(regs[1])?;
            let b = ctx.force_number(regs[2])?;
            ctx.set(regs[0],Value::InfiniteNumber(f(a,b)))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_num2ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[0])?,ctx.is_finite(regs[1])?) {
            (true, true) => {
                let a = ctx.force_finite_number(regs[0])?;
                let b = ctx.force_finite_number(regs[1])?;  
                let v = a.iter().zip(b.iter()).map(|(a,b)| f(*a,*b)).collect();
                ctx.set(regs[0],Value::FiniteNumber(v))?;
            },
            (true, false) => {
                let a = ctx.force_finite_number(regs[0])?;
                let b = ctx.force_infinite_number(regs[1])?;  
                let v = a.iter().map(|a| f(*a,b)).collect();
                ctx.set(regs[0],Value::FiniteNumber(v))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let a = ctx.force_infinite_number(regs[0])?;
                let b = ctx.force_infinite_number(regs[1])?;
                ctx.set(regs[0],Value::InfiniteNumber(f(a,b)))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

fn op_num3ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(f64,f64) -> f64 + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let a = ctx.force_finite_number(regs[1])?;
                let b = ctx.force_finite_number(regs[2])?;  
                let v = a.iter().zip(b.iter()).map(|(a,b)| f(*a,*b)).collect();
                ctx.set(regs[0],Value::FiniteNumber(v))?;
            },
            (true, false) => {
                let a = ctx.force_finite_number(regs[1])?;
                let b = ctx.force_infinite_number(regs[2])?;  
                let v = a.iter().map(|a| f(*a,b)).collect();
                ctx.set(regs[0],Value::FiniteNumber(v))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let a = ctx.force_infinite_number(regs[1])?;
                let b = ctx.force_infinite_number(regs[2])?;
                ctx.set(regs[0],Value::InfiniteNumber(f(a,b)))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

pub(super) fn op_max2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2(|a,b| a.max(b))
}

pub(super) fn op_max3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3(|a,b| a.max(b))
}

pub(super) fn op_min2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2(|a,b| a.min(b))
}

pub(super) fn op_min3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3(|a,b| a.min(b))
}

pub(super) fn op_max2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2s(|a,b| a.max(b))
}

pub(super) fn op_max3s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3s(|a,b| a.max(b))
}

pub(super) fn op_min2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2s(|a,b| a.min(b))
}

pub(super) fn op_min3s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3s(|a,b| a.min(b))
}

pub(super) fn op_max2ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2ss(|a,b| a.max(b))
}

pub(super) fn op_max3ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3ss(|a,b| a.max(b))
}

pub(super) fn op_min2ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num2ss(|a,b| a.min(b))
}

pub(super) fn op_min3ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    op_num3ss(|a,b| a.min(b))
}
