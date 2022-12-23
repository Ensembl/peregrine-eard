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
