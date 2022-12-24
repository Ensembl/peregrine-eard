use crate::model::constants::{FullConstant, Constant};

pub(crate) fn fold_add(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a+*b))])
    } else {
        None
    }
}

pub(crate) fn fold_sub(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a-*b))])
    } else {
        None
    }
}

pub(crate) fn fold_mul(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a**b))])
    } else {
        None
    }
}

pub(crate) fn fold_div(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a/ *b))])
    } else {
        None
    }
}

pub(crate) fn fold_gt(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Boolean(*a>*b))])
    } else {
        None
    }
}

pub(crate) fn fold_ge(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Boolean(*a>=*b))])
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
