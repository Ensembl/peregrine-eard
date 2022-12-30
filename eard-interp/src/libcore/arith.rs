use std::mem;

use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return, value::Value};

fn op_num2<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_number(regs[0])?;
        let b = ctx.force_number(regs[1])?;
        f(&mut a,b);
        ctx.set(regs[0],Value::Number(a))?;
        Ok(Return::Sync)
    }))
}

fn op_num3<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_number(regs[1])?;
        let b = ctx.force_number(regs[2])?;
        f(&mut a,b);
        ctx.set(regs[0],Value::Number(a))?;
        Ok(Return::Sync)
    }))
}

fn op_num2s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = mem::replace(ctx.force_finite_number_mut(regs[0])?,vec![]);
            let b = ctx.force_number(regs[1])?;
            for v in &mut a {
                f(v,b);
            }
           *ctx.force_finite_number_mut(regs[0])? = a;
        } else {
            let mut a = ctx.force_infinite_number(regs[0])?;
            let b = ctx.force_number(regs[1])?;
            f(&mut a,b);
            ctx.set(regs[0],Value::InfiniteNumber(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_num3s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = ctx.force_finite_number(regs[1])?.to_vec();
            let b = ctx.force_number(regs[2])?;
            for v in &mut a {
                f(v,b);
            }
            ctx.set(regs[0],Value::FiniteNumber(a))?;
        } else {
            let mut a = ctx.force_infinite_number(regs[1])?;
            let b = ctx.force_number(regs[2])?;
            f(&mut a,b);
            ctx.set(regs[0],Value::InfiniteNumber(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_num2ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[0])?,ctx.is_finite(regs[1])?) {
            (true, true) => {
                let mut a = mem::replace(ctx.force_finite_number_mut(regs[0])?,vec![]);
                let b = ctx.force_finite_number(regs[1])?;  
                for (x,y) in a.iter_mut().zip(b.iter()) {
                    f(x,*y);
                }
                *ctx.force_finite_number_mut(regs[0])? = a;
            },
            (true, false) => {
                let mut a = mem::replace(ctx.force_finite_number_mut(regs[0])?,vec![]);
                let b = ctx.force_infinite_number(regs[1])?;
                for v in &mut a {
                    f(v,b);
                }
               *ctx.force_finite_number_mut(regs[0])? = a;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let mut a = ctx.force_infinite_number(regs[0])?;
                let b = ctx.force_infinite_number(regs[1])?;
                f(&mut a,b);
                ctx.set(regs[0],Value::InfiniteNumber(a))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

fn op_num3ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64,f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let mut a = ctx.force_finite_number(regs[1])?.to_vec();
                let b = ctx.force_finite_number(regs[2])?;
                for (x,y) in a.iter_mut().zip(b.iter()) {
                    f(x,*y);
                }
                ctx.set(regs[0],Value::FiniteNumber(a))?;
            },
            (true, false) => {
                let mut a = ctx.force_finite_number(regs[1])?.to_vec();
                let b = ctx.force_infinite_number(regs[2])?;
                for x in a.iter_mut() {
                    f(x,b);
                }
                ctx.set(regs[0],Value::FiniteNumber(a))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let mut a = ctx.force_infinite_number(regs[1])?;
                let b = ctx.force_infinite_number(regs[2])?;
                f(&mut a,b);
                ctx.set(regs[0],Value::InfiniteNumber(a))?;        
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

op_binnum!(op_max2,op_max3,op_max2s,op_max3s,op_max2ss,op_max3ss,|a,b| *a = a.max(b));
op_binnum!(op_min2,op_min3,op_min2s,op_min3s,op_min2ss,op_min3ss,|a,b| *a = a.min(b));
op_binnum!(op_add2,op_add3,op_add2s,op_add3s,op_add2ss,op_add3ss,|a,b| *a += b);
op_binnum!(op_sub2,op_sub3,op_sub2s,op_sub3s,op_sub2ss,op_sub3ss,|a,b| *a -= b);
op_binnum!(op_mul2,op_mul3,op_mul2s,op_mul3s,op_mul2ss,op_mul3ss,|a,b| *a *= b);
op_binnum!(op_div2,op_div3,op_div2s,op_div3s,op_div2ss,op_div3ss,|a,b| *a /= b);
