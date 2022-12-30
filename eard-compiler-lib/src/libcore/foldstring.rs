use crate::model::constants::{FullConstant, Constant};
use super::util::{to_string, arm };

pub(crate) fn fold_join(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::String(sep)))),
            Some(Some(FullConstant::Finite(parts)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let Some(parts) = to_string(parts) {
            return Some(vec![FullConstant::Atomic(Constant::String(
                parts.join(sep)
            ))])
        }
    }
    None
}

pub(crate) fn fold_push_str(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(input))),
            Some(Some(FullConstant::Atomic(more)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let (Some(input),Some(more)) = (arm!(input,String),arm!(more,String)) {
            return Some(vec![FullConstant::Atomic(Constant::String(
                format!("{}{}",input,more)
            ))])
        }
    }
    if let (Some(Some(FullConstant::Infinite(input))),
            Some(Some(FullConstant::Atomic(more)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let (Some(input),Some(more)) = (arm!(input,String),arm!(more,String)) {
            return Some(vec![FullConstant::Infinite(Constant::String(
                format!("{}{}",input,more)
            ))])
        }
    }
    if let (Some(Some(FullConstant::Finite(input))),
            Some(Some(FullConstant::Atomic(more)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let (Some(input),Some(more)) = (to_string(input),arm!(more,String)) {
            return Some(vec![FullConstant::Finite(
                input.iter().map(|s| {
                    Constant::String(format!("{}{}",s,more))
                }).collect::<Vec<_>>()
            )])
        }
    }    
    None
}

pub(crate) fn fold_split(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(sep))),
            Some(Some(FullConstant::Atomic(input)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let (Some(input),Some(sep)) = (arm!(input,String),arm!(sep,String)) {
            return Some(vec![FullConstant::Finite(
                input.split(sep).map(|s| Constant::String(s.to_string())).collect()
            )])
        }
    }
    None
}

fn apply_template(template: &str, values: &[String]) -> Option<String> {
    let mut out = String::new();
    let mut lone_obrace = false;
    let mut lone_cbrace = false;
    let mut value = None;
    for c in template.chars() {
        if lone_obrace {
            if c == '{' { out.push('{'); }
            else if c.is_digit(10) { 
                value = c.to_string().parse::<usize>().ok();
                if value.is_none() { return None; }
            }
            else { return None; }
            lone_obrace = false;
        } else if lone_cbrace {
            if c == '}' { out.push('}'); }
            else { return None; }
            lone_cbrace = false;
        } else if value.is_some() {
            if c == '}' { out.push_str(&values[value.unwrap()]); value = None; }
            else if c.is_digit(10) {
                let more = c.to_string().parse::<usize>().ok();
                if more.is_none() { return None; }
                value = Some(value.unwrap()*10+more.unwrap());
            }
        } else if c == '{' {
            lone_obrace = true;
        } else if c == '}' {
            lone_cbrace = true;
        } else {
            out.push(c);
        }
    }
    if lone_obrace || lone_cbrace || value.is_some() { return None; }
    return Some(out);
}

pub(crate) fn fold_template(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::String(template)))),
            Some(Some(FullConstant::Finite(values)))) = 
               (inputs.get(0),inputs.get(1)) {
        if let Some(values) = to_string(values) {
            if let Some(out) = apply_template(template,&values) {
                return Some(vec![FullConstant::Atomic(Constant::String(out))]);
            }
        }
    }
    None
}

fn format(input: &Constant) -> String {
    match input {
        Constant::Number(n) => format!("{}",n.0),
        Constant::String(s) => format!("{:?}",s),
        Constant::Boolean(b) => format!("{:?}",b),
    }
}

pub(crate) fn fold_format(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let Some(Some(value)) = inputs.get(0) {
        Some(vec![FullConstant::Atomic(Constant::String(match value {
            FullConstant::Atomic(a) => format(a),
            FullConstant::Finite(s) => {
                format!("[{}]",s.iter().map(|a| format(a)).collect::<Vec<_>>().join(","))
            },
            FullConstant::Infinite(a) => {
                format!("[{},...]",format(a))
            }
        }))])
    } else {
        None
    }
}
