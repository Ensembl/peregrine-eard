use std::mem;
use crate::controller::{operation::Return, value::Value, globalcontext::{GlobalContext,GlobalBuildContext}};

fn op_bool2<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_boolean(regs[0])?;
        let b = ctx.force_boolean(regs[1])?;
        f(&mut a,b);
        ctx.set(regs[0],Value::Boolean(a))?;
        Ok(Return::Sync)
    }))
}

fn op_bool3<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        let mut a = ctx.force_boolean(regs[1])?;
        let b = ctx.force_boolean(regs[2])?;
        f(&mut a,b);
        ctx.set(regs[0],Value::Boolean(a))?;
        Ok(Return::Sync)
    }))
}

fn op_bool2_s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let mut a = mem::replace(ctx.force_finite_boolean_mut(regs[0])?,vec![]);
            let b = ctx.force_boolean(regs[1])?;
            for v in &mut a {
                f(v,b);
            }
           *ctx.force_finite_boolean_mut(regs[0])? = a;
        } else {
            let mut a = ctx.force_infinite_boolean(regs[0])?;
            let b = ctx.force_boolean(regs[1])?;
            f(&mut a,b);
            ctx.set(regs[0],Value::InfiniteBoolean(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_bool3_s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[1])? {
            let mut a = ctx.force_finite_boolean(regs[1])?.to_vec();
            let b = ctx.force_boolean(regs[2])?;
            for v in &mut a {
                f(v,b);
            }
            ctx.set(regs[0],Value::FiniteBoolean(a))?;
        } else {
            let mut a = ctx.force_infinite_boolean(regs[1])?;
            let b = ctx.force_boolean(regs[2])?;
            f(&mut a,b);
            ctx.set(regs[0],Value::InfiniteBoolean(a))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_bool2_ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[0])?,ctx.is_finite(regs[1])?) {
            (true, true) => {
                let mut a = mem::replace(ctx.force_finite_boolean_mut(regs[0])?,vec![]);
                let b = ctx.force_finite_boolean(regs[1])?;  
                for (x,y) in a.iter_mut().zip(b.iter()) {
                    f(x,*y);
                }
                *ctx.force_finite_boolean_mut(regs[0])? = a;
            },
            (true, false) => {
                let mut a = mem::replace(ctx.force_finite_boolean_mut(regs[0])?,vec![]);
                let b = ctx.force_infinite_boolean(regs[1])?;
                for v in &mut a {
                    f(v,b);
                }
               *ctx.force_finite_boolean_mut(regs[0])? = a;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let mut a = ctx.force_infinite_boolean(regs[0])?;
                let b = ctx.force_infinite_boolean(regs[1])?;
                f(&mut a,b);
                ctx.set(regs[0],Value::InfiniteBoolean(a))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

fn op_bool3_ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&mut bool,bool) + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let mut a = ctx.force_finite_boolean(regs[1])?.to_vec();
                let b = ctx.force_finite_boolean(regs[2])?;
                for (x,y) in a.iter_mut().zip(b.iter()) {
                    f(x,*y);
                }
                ctx.set(regs[0],Value::FiniteBoolean(a))?;
            },
            (true, false) => {
                let mut a = ctx.force_finite_boolean(regs[1])?.to_vec();
                let b = ctx.force_infinite_boolean(regs[2])?;
                for x in a.iter_mut() {
                    f(x,b);
                }
                ctx.set(regs[0],Value::FiniteBoolean(a))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let mut a = ctx.force_infinite_boolean(regs[1])?;
                let b = ctx.force_infinite_boolean(regs[2])?;
                f(&mut a,b);
                ctx.set(regs[0],Value::InfiniteBoolean(a))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

macro_rules! op_binbool {
    ($op2:ident,$op3:ident,$op2s:ident,$op3s:ident,$op2ss:ident,$op3ss:ident,$cb:expr) => {
        #[allow(unused)]
        pub(super) fn $op2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2($cb)
        }

        pub(super) fn $op3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3($cb)
        }

        #[allow(unused)]
        pub(super) fn $op2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2_s($cb)
        }

        pub(super) fn $op3s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3_s($cb)
        }

        #[allow(unused)]
        pub(super) fn $op2ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2_ss($cb)
        }

        pub(super) fn $op3ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3_ss($cb)
        }
    };
}

op_binbool!(op_eq2_bool,op_eq3_bool,op_eq2_bool_s,op_eq3_bool_s,op_eq2_bool_ss,op_eq3_bool_ss,|a,b| *a = *a==b);
op_binbool!(op_and2,op_and3,op_and2_s,op_and3_s,op_and2_ss,op_and3_ss,|a,b| *a = *a && b);
op_binbool!(op_or2,op_or3,op_or2_s,op_or3_s,op_or2_ss,op_or3_ss,|a,b| *a = *a || b);
