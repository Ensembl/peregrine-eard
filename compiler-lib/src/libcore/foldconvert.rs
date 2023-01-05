use ordered_float::OrderedFloat;

use crate::model::constants::{FullConstant, Constant};
use super::util::seq_flex_un;

fn to_boolean(c: &Constant) -> Constant {
    Constant::Boolean(match c {
        Constant::Number(n) => n.0 != 0.,
        Constant::String(s) => s != "",
        Constant::Boolean(b) => *b
    })
}

fn to_string(c: &Constant) -> Constant {
    Constant::String(match c {
        Constant::Number(n) => n.0.to_string(),
        Constant::String(s) => s.to_string(),
        Constant::Boolean(b) => (if *b { "true" } else { "false" }).to_string()
    })
}

fn to_number(c: &Constant) -> Constant {
    Constant::Number(OrderedFloat(match c {
        Constant::Number(n) => n.0,
        Constant::String(s) => s.parse::<f64>().unwrap_or(0.),
        Constant::Boolean(b) => (if *b { 1. } else { 0. }),
    }))
}

pub(crate) fn fold_to_boolean(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(value)) = inputs.get(0) {
        Some(vec![seq_flex_un!(value,to_boolean)])
    } else {
        None
    }
}

pub(crate) fn fold_to_string(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(value)) = inputs.get(0) {
        Some(vec![seq_flex_un!(value,to_string)])
    } else {
        None
    }
}

pub(crate) fn fold_to_number(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(value)) = inputs.get(0) {
        Some(vec![seq_flex_un!(value,to_number)])
    } else {
        None
    }
}
