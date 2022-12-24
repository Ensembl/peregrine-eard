use ordered_float::OrderedFloat;

use crate::model::constants::{FullConstant, Constant};

pub(super) fn fold_infseq(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Atomic(x))) = inputs.get(0) {
        Some(vec![FullConstant::Infinite(x.clone())])
    } else {
        None
    }
}

pub(super) fn fold_finseq(_: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    Some(vec![FullConstant::Finite(vec![])])
}

pub(super) fn fold_push(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Finite(x))),
            Some(Some(FullConstant::Atomic(y)))) = 
               (inputs.get(0),inputs.get(1)) {
        let mut z = x.clone();
        z.push(y.clone());
        Some(vec![FullConstant::Finite(z)])
    } else {
        None
    }
}

pub(super) fn fold_length(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Finite(x))) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::Number(OrderedFloat(x.len() as f64)))])
    } else if let Some(Some(FullConstant::Infinite(_))) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::Number(OrderedFloat(-1.)))])
    } else {
        None
    }
}

fn total(input: &[Constant]) -> i64 {
    let mut total = 0;
    for v in input {
        if let Constant::Number(OrderedFloat(x)) = v {
            if *x < 0. || x.fract() != 0.0 { return -1; }
            total += *x as i64;
        } else {
            return -1;
        }
    }
    total
}

pub(super) fn fold_total(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Finite(x))) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::Number(OrderedFloat(total(x) as f64)))])
    } else {
        None
    }
}

fn bound(input: &[Constant]) -> i64 {
    let mut max = 0;
    for v in input {
        if let Constant::Number(OrderedFloat(x)) = v {
            if *x < 0. || x.fract() != 0.0 { return -1; }
            max = max.max(*x as i64);
        } else {
            return -1;
        }
    }
    max
}

pub(super) fn fold_bound(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Finite(x))) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::Number(OrderedFloat(bound(x) as f64)))])
    } else {
        None
    }
}

pub(super) fn fold_if(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(FullConstant::Atomic(Constant::Boolean(b)))) = inputs.get(0) {
        let idx = if *b { 1 } else { 2 };
        if let Some(Some(x)) = inputs.get(idx) {
            return Some(vec![x.clone()]);
        }
    }
    None
}
