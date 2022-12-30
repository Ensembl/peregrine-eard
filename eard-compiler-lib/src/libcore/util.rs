use crate::model::constants::{Constant, FullConstant};

macro_rules! arm {
    ($ex:expr,$arm:tt) => {
        match $ex { Constant::$arm(x) => Some(x), _ => None }
    };
}

pub(crate) use arm;

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

use ordered_float::OrderedFloat;
pub(crate) use seq_flex;

macro_rules! seq_flex_un {
    ($a:expr,$f0:ident,$f1:ident) => {
        match ($a) {
            FullConstant::Atomic(a) => FullConstant::Atomic($f0(a)),
            FullConstant::Finite(a) => FullConstant::Finite($f1(a)),
            FullConstant::Infinite(a) => FullConstant::Infinite($f0(a)),
        }
    };

    ($a:expr,$f0:ident) => {
        match ($a) {
            FullConstant::Atomic(a) => FullConstant::Atomic($f0(&a)),
            FullConstant::Finite(s) => FullConstant::Finite(s.iter().map(|a| $f0(a)).collect() ),
            FullConstant::Infinite(a) => FullConstant::Infinite($f0(&a)),
        }
    };
}

pub(crate) use seq_flex_un;

pub(crate) fn to_usize(input: &[Constant]) -> Option<Vec<usize>> {
    input.iter().map(|idx| {
        match idx {
            Constant::Number(n) => Some(n.0 as usize),
            _ => None
        }
    }).collect::<Option<Vec<_>>>()
}

pub(crate) fn to_string(input: &[Constant]) -> Option<Vec<String>> {
    input.iter().map(|idx| {
        match idx {
            Constant::String(n) => Some(n.to_string()),
            _ => None
        }
    }).collect::<Option<Vec<_>>>()
}

pub(crate) fn to_boolean(input: &[Constant]) -> Option<Vec<bool>> {
    input.iter().map(|idx| {
        match idx {
            Constant::Boolean(b) => Some(*b),
            _ => None
        }
    }).collect::<Option<Vec<_>>>()
}

pub(crate) fn fold_num_bin<F>(inputs: &[Option<FullConstant>], cb: F) -> Option<Vec<FullConstant>>
        where F: Fn(f64,f64) -> f64 {
    if let (Some(Some(FullConstant::Atomic(a))),
            Some(Some(FullConstant::Atomic(b)))) = (inputs.get(0),inputs.get(1)) {
        if let (Some(a),Some(b)) = (arm!(a,Number),arm!(b,Number)) {
            return Some(vec![FullConstant::Atomic(Constant::Number(OrderedFloat(cb(a.0,b.0))))]);
        }
    } else if let (Some(Some(FullConstant::Infinite(a))),
                   Some(Some(FullConstant::Atomic(b)))) = (inputs.get(0),inputs.get(1)) {
        if let (Some(a),Some(b)) = (arm!(a,Number),arm!(b,Number)) {
            return Some(vec![FullConstant::Infinite(Constant::Number(OrderedFloat(cb(a.0,b.0))))]);
        }
    } else if let (Some(Some(FullConstant::Finite(a))),
                   Some(Some(FullConstant::Atomic(Constant::Number(b))))) = (inputs.get(0),inputs.get(1)) {
        let out = a.iter().map(|c| arm!(c,Number)).collect::<Option<Vec<_>>>()?;
        let z = out.iter().map(|v| Constant::Number(OrderedFloat(cb(v.0,b.0)))).collect::<Vec<_>>();
        return Some(vec![FullConstant::Finite(z)]);
    } else if let (Some(Some(FullConstant::Finite(a))),
                   Some(Some(FullConstant::Infinite(Constant::Number(b))))) = (inputs.get(0),inputs.get(1)) {
        let out = a.iter().map(|c| arm!(c,Number)).collect::<Option<Vec<_>>>()?;
        let z = out.iter().map(|v| Constant::Number(OrderedFloat(cb(v.0,b.0)))).collect::<Vec<_>>();
        return Some(vec![FullConstant::Finite(z)]);
    } else if let (Some(Some(FullConstant::Finite(a))),
                   Some(Some(FullConstant::Finite(b)))) = (inputs.get(0),inputs.get(1)) {
        let a = a.iter().map(|c| arm!(c,Number)).collect::<Option<Vec<_>>>()?;
        let b = b.iter().map(|c| arm!(c,Number)).collect::<Option<Vec<_>>>()?;
        let z = a.iter().zip(b.iter())
            .map(|(a,b)| Constant::Number(OrderedFloat(cb(a.0,b.0))))
            .collect::<Vec<_>>();
        return Some(vec![FullConstant::Finite(z)]);
    }
    None
}
