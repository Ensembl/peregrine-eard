use crate::model::constants::Constant;

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
