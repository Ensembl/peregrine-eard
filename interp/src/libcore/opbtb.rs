/* opbtb, ie bool -> bool */

use std::mem;
use crate::controller::{globalcontext::{GlobalBuildContext, GlobalContext}, operation::Return, value::Value};

fn op_ubool1<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_boolean(regs[0])?;
        f(&mut a);
        ctx.set(regs[0],Value::Boolean(a))?;
        Ok(Return::Sync)
    }))
}

fn op_ubool2<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_boolean(regs[1])?;
        f(&mut a);
        ctx.set(regs[0],Value::Boolean(a))?;
        Ok(Return::Sync)
    }))
}

fn op_ubool1s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = mem::replace(ctx.force_finite_boolean_mut(regs[0])?,vec![]);
            for v in &mut a {
                f(v);
            }
           *ctx.force_finite_boolean_mut(regs[0])? = a;
        } else {
            let mut a = ctx.force_infinite_boolean(regs[0])?;
            f(&mut a);
            ctx.set(regs[0],Value::InfiniteBoolean(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_ubool2s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let mut a = ctx.force_finite_boolean(regs[1])?.to_vec();
            for v in &mut a {
                f(v);
            }
            ctx.set(regs[0],Value::FiniteBoolean(a))?;
        } else {
            let mut a = ctx.force_infinite_boolean(regs[1])?;
            f(&mut a);
            ctx.set(regs[0],Value::InfiniteBoolean(a))?;    
        }
        Ok(Return::Sync)
    }))
}

macro_rules! op_ubool {
    ($op1:ident,$op2:ident,$op1s:ident,$op2s:ident,$cb:expr) => {
        #[allow(unused)]
        pub(super) fn $op1(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_ubool1($cb)
        }

        pub(super) fn $op2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_ubool2($cb)
        }

        #[allow(unused)]
        pub(super) fn $op1s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_ubool1s($cb)
        }

        pub(super) fn $op2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_ubool2s($cb)
        }
    };
}

op_ubool!(op_not1,op_not2,op_not1s,op_not2s,|a| *a = !*a);
