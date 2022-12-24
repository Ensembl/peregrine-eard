use crate::model::constants::{FullConstant, Constant};
use ordered_float::OrderedFloat;

macro_rules! seq_flex {
    ($a:expr,$b:expr,$f0:ident,$f1:ident,$f2:ident) => {
        match ($a,$b) {
            (FullConstant::Atomic(a), FullConstant::Atomic(b)) => FullConstant::Atomic($f0(a,b)),
            (FullConstant::Atomic(a), FullConstant::Finite(b)) => FullConstant::Finite($f1(b,a)),
            (FullConstant::Atomic(a), FullConstant::Infinite(b)) => FullConstant::Infinite($f0(a,b)),
            (FullConstant::Finite(a), FullConstant::Atomic(b)) => FullConstant::Finite($f1(a,b)),
            (FullConstant::Finite(a), FullConstant::Finite(b)) => FullConstant::Finite($f2(a,b)),
            (FullConstant::Finite(a), FullConstant::Infinite(b)) => FullConstant::Finite($f1(a,b)),
            (FullConstant::Infinite(a), FullConstant::Atomic(b)) => FullConstant::Infinite($f0(a,b)),
            (FullConstant::Infinite(a), FullConstant::Finite(b)) => FullConstant::Finite($f1(b,a)),
            (FullConstant::Infinite(a), FullConstant::Infinite(b)) => FullConstant::Infinite($f0(a,b)),
        }
    };
}

macro_rules! seq_flex_un {
    ($a:expr,$f0:ident,$f1:ident) => {
        match ($a) {
            FullConstant::Atomic(a) => FullConstant::Atomic($f0(a)),
            FullConstant::Finite(a) => FullConstant::Finite($f1(a)),
            FullConstant::Infinite(a) => FullConstant::Infinite($f0(a)),
        }
    };
}

macro_rules! arm {
    ($ex:expr,$arm:tt) => {
        match $ex { Constant::$arm(x) => Some(x), _ => None }
    };
}

macro_rules! number_pred {
    ($pred:expr,$a:expr,$b:expr) => {{
        fn f0(a: &Constant, b: &Constant) -> Constant {
            Constant::Boolean(($pred)(arm!(a,Number),arm!(b,Number)))
        }
        
        fn f1(a: &[Constant], b: &Constant) -> Vec<Constant> {
            a.iter().map(|a| f0(a,b)).collect()
        }
        
        fn f2(a: &[Constant], b: &[Constant]) -> Vec<Constant> {
            a.iter().zip(b.iter()).map(|(a,b)| f0(a,b)).collect()
        }

        seq_flex!($a,$b,f0,f1,f2)
    }};
}

macro_rules! number_bin {
    ($op:expr,$a:expr,$b:expr) => {{
        fn f0(a: &Constant, b: &Constant) -> Constant {
            if let (Some(a),Some(b)) = (arm!(a,Number),arm!(b,Number)) {
                Constant::Number(OrderedFloat(($op)(a.0,b.0)))
            } else {
                Constant::Number(OrderedFloat(f64::NAN))
            }
        }
        
        fn f1(a: &[Constant], b: &Constant) -> Vec<Constant> {
            a.iter().map(|a| f0(a,b)).collect()
        }
        
        fn f2(a: &[Constant], b: &[Constant]) -> Vec<Constant> {
            a.iter().zip(b.iter()).map(|(a,b)| f0(a,b)).collect()
        }

        seq_flex!($a,$b,f0,f1,f2)
    }};
}

macro_rules! number_un {
    ($op:expr,$a:expr) => {{
        fn f0(a: &Constant) -> Constant {
            if let Some(a) = arm!(a,Number) {
                Constant::Number(OrderedFloat(($op)(a.0)))
            } else {
                Constant::Number(OrderedFloat(f64::NAN))
            }
        }
        
        fn f1(a: &[Constant]) -> Vec<Constant> {
            a.iter().map(|a| f0(a)).collect()
        }
        
        seq_flex_un!($a,f0,f1)
    }};
}

macro_rules! logic_bin {
    ($op:expr,$a:expr,$b:expr) => {{
        fn f0(a: &Constant, b: &Constant) -> Constant {
            if let (Some(a),Some(b)) = (arm!(a,Boolean),arm!(b,Boolean)) {
                Constant::Boolean(($op)(*a,*b))
            } else {
                Constant::Boolean(false)
            }
        }
        
        fn f1(a: &[Constant], b: &Constant) -> Vec<Constant> {
            a.iter().map(|a| f0(a,b)).collect()
        }
        
        fn f2(a: &[Constant], b: &[Constant]) -> Vec<Constant> {
            a.iter().zip(b.iter()).map(|(a,b)| f0(a,b)).collect()
        }

        seq_flex!($a,$b,f0,f1,f2)
    }};
}

pub(crate) fn fold_minus(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(a)) = inputs.get(0) {
        Some(vec![number_un!(|a: f64| -a,a)])
    } else {
        None
    }
}

pub(crate) fn fold_add(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_bin!(|a,b| a+b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_sub(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_bin!(|a,b| a-b,a,b)])
    } else {
        None
    }
}
pub(crate) fn fold_mul(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_bin!(|a,b| a*b,a,b)])
    } else {
        None
    }
}
pub(crate) fn fold_div(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_bin!(|a,b| a/b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_and(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![logic_bin!(|a: bool,b| a && b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_or(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![logic_bin!(|a: bool,b| a || b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_gt(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_pred!(|a,b| a>b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_ge(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_pred!(|a,b| a>=b,a,b)])
    } else {
        None
    }
}

pub(crate) fn fold_not(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Atomic(Constant::Boolean(a)))) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::Boolean(!*a))])
    } else if let Some(Some(FullConstant::Finite(seq))) = inputs.get(0) {
        Some(vec![FullConstant::Finite(seq.iter().map(|c| match c {
            Constant::Boolean(x) => Constant::Boolean(!*x),
            _ => Constant::Boolean(false)
        }).collect::<Vec<_>>())])
    } else if let Some(Some(FullConstant::Infinite(Constant::Boolean(a)))) = inputs.get(0) {
        Some(vec![FullConstant::Infinite(Constant::Boolean(!*a))])
    } else {
        None
    }
}

pub(crate) fn fold_eq(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(a)),Some(Some(b))) = (inputs.get(0),inputs.get(1)) {
        Some(vec![number_pred!(|a,b| a==b,a,b)])
    } else {
        None
    }
}
