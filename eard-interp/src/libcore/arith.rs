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

macro_rules! op_binnum {
    ($op2:ident,$op3:ident,$op2s:ident,$op3s:ident,$op2ss:ident,$op3ss:ident,$cb:expr) => {
        pub(super) fn $op2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num2($cb)
        }
        
        pub(super) fn $op3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num3($cb)
        }
        
        pub(super) fn $op2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num2s($cb)
        }
        
        pub(super) fn $op3s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num3s($cb)
        }
        
        pub(super) fn $op2ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num2ss($cb)
        }
        
        pub(super) fn $op3ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_num3ss($cb)
        }
    };
}

op_binnum!(op_max2,op_max3,op_max2s,op_max3s,op_max2ss,op_max3ss,|a,b| a.max(b));
op_binnum!(op_min2,op_min3,op_min2s,op_min3s,op_min2ss,op_min3ss,|a,b| a.min(b));
op_binnum!(op_add2,op_add3,op_add2s,op_add3s,op_add2ss,op_add3ss,|a,b| a+b);
op_binnum!(op_sub2,op_sub3,op_sub2s,op_sub3s,op_sub2ss,op_sub3ss,|a,b| a-b);
