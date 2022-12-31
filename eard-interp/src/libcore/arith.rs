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

fn op_numbool<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(f64,f64) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        let a = ctx.force_number(regs[1])?;
        let b = ctx.force_number(regs[2])?;
        ctx.set(regs[0],Value::Boolean(f(a,b)))?;
        Ok(Return::Sync)
    }))
}

fn op_numbool_s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(f64,f64) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let a = ctx.force_finite_number(regs[1])?;
            let b = ctx.force_number(regs[2])?;
            ctx.set(regs[0],Value::FiniteBoolean(a.iter().map(|v| f(*v,b)).collect()))?;
        } else {
            let a = ctx.force_infinite_number(regs[1])?;
            let b = ctx.force_number(regs[2])?;
            ctx.set(regs[0],Value::InfiniteBoolean(f(a,b)))?;    
        }
        Ok(Return::Sync)
    }))
}

fn op_numbool_ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(f64,f64) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let a = ctx.force_finite_number(regs[1])?;
                let b = ctx.force_finite_number(regs[2])?;
                let v = a.iter().zip(b.iter()).map(|(a,b)| f(*a,*b)).collect();
                ctx.set(regs[0],Value::FiniteBoolean(v))?;
            },
            (true, false) => {
                let a = ctx.force_finite_number(regs[1])?;
                let b = ctx.force_infinite_number(regs[2])?;
                let v = a.iter().map(|a| f(*a,b)).collect();
                ctx.set(regs[0],Value::FiniteBoolean(v))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let a = ctx.force_infinite_number(regs[1])?;
                let b = ctx.force_infinite_number(regs[2])?;
                ctx.set(regs[0],Value::InfiniteBoolean(f(a,b)))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

fn op_strbool<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&str,&str) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        let a = ctx.force_string(regs[1])?;
        let b = ctx.force_string(regs[2])?;
        ctx.set(regs[0],Value::Boolean(f(a,b)))?;
        Ok(Return::Sync)
    }))
}

fn op_strbool_s<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String>
        where F: Fn(&str,&str) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        if ctx.is_finite(regs[0])? {
            let a = ctx.force_finite_string(regs[1])?;
            let b = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::FiniteBoolean(a.iter().map(|v| f(v,b)).collect()))?;
        } else {
            let a = ctx.force_infinite_string(regs[1])?;
            let b = ctx.force_string(regs[2])?;
            ctx.set(regs[0],Value::InfiniteBoolean(f(a,b)))?;
        }
        Ok(Return::Sync)
    }))
}

fn op_strbool_ss<F>(f: F) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> 
        where F: Fn(&str,&str) -> bool + 'static {
    Ok(Box::new(move |ctx,regs| {
        match (ctx.is_finite(regs[1])?,ctx.is_finite(regs[2])?) {
            (true, true) => {
                let a = ctx.force_finite_string(regs[1])?;
                let b = ctx.force_finite_string(regs[2])?;
                let v = a.iter().zip(b.iter()).map(|(a,b)| f(a,b)).collect();
                ctx.set(regs[0],Value::FiniteBoolean(v))?;
            },
            (true, false) => {
                let a = ctx.force_finite_string(regs[1])?;
                let b = ctx.force_infinite_string(regs[2])?;
                let v = a.iter().map(|a| f(a,b)).collect();
                ctx.set(regs[0],Value::FiniteBoolean(v))?;
            },
            (false, true) => {
                return Err(format!("length mismatch"));
            },
            (false, false) => {
                let a = ctx.force_infinite_string(regs[1])?;
                let b = ctx.force_infinite_string(regs[2])?;
                ctx.set(regs[0],Value::InfiniteBoolean(f(a,b)))?;        
            }
        }
        Ok(Return::Sync)
    }))
}

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
        if ctx.is_finite(regs[0])? {
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

macro_rules! op_binnumbool {
    ($op:ident,$ops:ident,$opss:ident,$cb:expr) => {
        pub(super) fn $op(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_numbool($cb)
        }
                
        pub(super) fn $ops(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_numbool_s($cb)
        }
                
        pub(super) fn $opss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_numbool_ss($cb)
        }
    };
}

macro_rules! op_binstrbool {
    ($op:ident,$ops:ident,$opss:ident,$cb:expr) => {
        pub(super) fn $op(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_strbool($cb)
        }
                
        pub(super) fn $ops(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_strbool_s($cb)
        }
                
        pub(super) fn $opss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_strbool_ss($cb)
        }
    };
}

macro_rules! op_binbool {
    ($op2:ident,$op3:ident,$op2s:ident,$op3s:ident,$op2ss:ident,$op3ss:ident,$cb:expr) => {
        pub(super) fn $op2(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2($cb)
        }

        pub(super) fn $op3(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3($cb)
        }
                
        pub(super) fn $op2s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2_s($cb)
        }

        pub(super) fn $op3s(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3_s($cb)
        }

        pub(super) fn $op2ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool2_ss($cb)
        }

        pub(super) fn $op3ss(_gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
            op_bool3_ss($cb)
        }
    };
}


op_binnum!(op_max2,op_max3,op_max2s,op_max3s,op_max2ss,op_max3ss,|a,b| *a = a.max(b));
op_binnum!(op_min2,op_min3,op_min2s,op_min3s,op_min2ss,op_min3ss,|a,b| *a = a.min(b));
op_binnum!(op_add2,op_add3,op_add2s,op_add3s,op_add2ss,op_add3ss,|a,b| *a += b);
op_binnum!(op_sub2,op_sub3,op_sub2s,op_sub3s,op_sub2ss,op_sub3ss,|a,b| *a -= b);
op_binnum!(op_mul2,op_mul3,op_mul2s,op_mul3s,op_mul2ss,op_mul3ss,|a,b| *a *= b);
op_binnum!(op_div2,op_div3,op_div2s,op_div3s,op_div2ss,op_div3ss,|a,b| *a /= b);
op_binnum!(op_mod2,op_mod3,op_mod2s,op_mod3s,op_mod2ss,op_mod3ss,|a,b| *a %= b);

op_binnumbool!(op_gt,op_gt_s,op_gt_ss,|a,b| a>b);
op_binnumbool!(op_ge,op_ge_s,op_ge_ss,|a,b| a>=b);
op_binnumbool!(op_eq_num,op_eq_num_s,op_eq_num_ss,|a,b| a==b);
op_binstrbool!(op_eq_str,op_eq_str_s,op_eq_str_ss,|a,b| a==b);
op_binbool!(op_eq2_bool,op_eq3_bool,op_eq2_bool_s,op_eq3_bool_s,op_eq2_bool_ss,op_eq3_bool_ss,|a,b| *a = *a==b);
op_binbool!(op_and2,op_and3,op_and2_s,op_and3_s,op_and2_ss,op_and3_ss,|a,b| *a = *a && b);
op_binbool!(op_or2,op_or3,op_or2_s,op_or3_s,op_or2_ss,op_or3_ss,|a,b| *a = *a || b);
