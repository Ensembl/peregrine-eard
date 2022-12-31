/* ntn number -> number */

use std::mem;
use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return, value::Value};

fn op_unum1<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_number(regs[0])?;
        f(&mut a);
        ctx.set(regs[0],Value::Number(a))?;
        Ok(Return::Sync)
    }))
}

fn op_unum2<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_number(regs[1])?;
        f(&mut a);
        ctx.set(regs[0],Value::Number(a))?;
        Ok(Return::Sync)
    }))
}

fn op_unum1s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = mem::replace(ctx.force_finite_number_mut(regs[0])?,vec![]);
            for v in &mut a {
                f(v);
            }
           *ctx.force_finite_number_mut(regs[0])? = a;
        } else {
            let mut a = ctx.force_infinite_number(regs[0])?;
            f(&mut a);
            ctx.set(regs[0],Value::InfiniteNumber(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_unum2s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut f64) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = ctx.force_finite_number(regs[1])?.to_vec();
            for v in &mut a {
                f(v);
            }
            ctx.set(regs[0],Value::FiniteNumber(a))?;
        } else {
            let mut a = ctx.force_infinite_number(regs[1])?;
            f(&mut a);
            ctx.set(regs[0],Value::InfiniteNumber(a))?;    
        }
        Ok(Return::Sync)
    }))
}

macro_rules! op_unum {
    ($op1:ident,$op2:ident,$op1s:ident,$op2s:ident,$cb:expr) => {
        #[allow(unused)]
        pub(super) fn $op1(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_unum1($cb)
        }

        pub(super) fn $op2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_unum2($cb)
        }

        #[allow(unused)]
        pub(super) fn $op1s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_unum1s($cb)
        }

        pub(super) fn $op2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_unum2s($cb)
        }
    };
}

op_unum!(op_neg1,op_neg2,op_neg1s,op_neg2s,|a| *a = -*a);
