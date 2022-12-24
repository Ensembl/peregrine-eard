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
