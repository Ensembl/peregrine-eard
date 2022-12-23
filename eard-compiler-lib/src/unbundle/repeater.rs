use crate::frontend::buildtree::{BTProcCall, BTLValue, BTExpression, Variable};
use crate::frontend::femodel::OrBundleRepeater;

fn list_lhs_repeaters(rets: &[OrBundleRepeater<BTLValue>]) -> Vec<String> {
    rets.iter().filter_map(|ret| {
        match ret {
            OrBundleRepeater::Repeater(r)=> { Some(r.to_string()) },
            _ => None
        }
    }).collect()
}

fn list_rhs_repeaters(args: &[OrBundleRepeater<BTExpression>]) -> Result<Vec<String>,String> {
    let mut repeaters = vec![];
    for arg in args {
        match arg {
            OrBundleRepeater::Normal(BTExpression::Function(f)) => {
                repeaters.append(&mut list_rhs_repeaters(&f.args)?);
            },
            OrBundleRepeater::Repeater(r) => { repeaters.push(r.to_string()) },
            _ => {}
        }
    }
    Ok(repeaters)
}

pub(super) fn find_repeater_arguments(call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<Option<(String,String)>,String> {
    /* A good repeater has a single use of the repeater glyph (**) on each side and has a
     * single lhs variable.
     */
    let lhs = match &call.rets {
        Some(x) => {
            let lhs_repeaters = list_lhs_repeaters(call.rets.as_ref().unwrap_or(&vec![]));
            if lhs_repeaters.len() > 1 {
                return Err("too many repeaters in lvalue".to_string());
            } else if lhs_repeaters.len() == 0 {
                None
            } else if call.rets.as_ref().unwrap_or(&vec![]).len() != 1 {
                return Err("repeater must be the only result in lvalue".to_string());
            } else {
                match &x[0] {
                    OrBundleRepeater::Repeater(r) => { Some(r) },
                    _ => { None } // lhs is not repeater
                }
            }
        },
        _ => { None }
    };
    let rhs_repeaters = list_rhs_repeaters(&call.args)?;
    if rhs_repeaters.len() > 1 {
        return Err("too many repeaters in rvalue".to_string());
    }
    let rhs = rhs_repeaters.first();
    Ok(match (lhs,rhs) {
        (Some(lhs),Some(rhs)) => Some((lhs.to_string(),rhs.to_string())),
        (None,None) => None,
        _ => { return Err("must be single repeater on left and right".to_string()); }
    })
}

fn rewrite_lvalue(ret: &OrBundleRepeater<BTLValue>, variable: &Variable) -> OrBundleRepeater<BTLValue> {
    match ret {
        OrBundleRepeater::Repeater(_) => OrBundleRepeater::Normal(BTLValue::Variable(variable.clone())),
        x => x.clone()
    }
}

fn rewrite_rvalue(arg: &OrBundleRepeater<BTExpression>, variable: &Variable) -> OrBundleRepeater<BTExpression> {
    match arg {
        OrBundleRepeater::Repeater(_) => OrBundleRepeater::Normal(BTExpression::Variable(variable.clone())),
        OrBundleRepeater::Normal(BTExpression::Function(func)) => {
            let mut func = func.clone();
            func.args = func.args.iter().map(|arg| rewrite_rvalue(arg,variable)).collect();
            OrBundleRepeater::Normal(BTExpression::Function(func))
        },
        x => x.clone()
    }
}

pub(super) fn rewrite_repeater(proc: &BTProcCall<OrBundleRepeater<BTLValue>>, left: &Variable, right: &Variable) -> Result<BTProcCall<OrBundleRepeater<BTLValue>>,String> {
    let mut out = proc.clone();
    out.rets = proc.rets.as_ref().map(|rets|
        rets.iter().map(|ret| rewrite_lvalue(ret,left)).collect::<Vec<_>>()
    );
    out.args = proc.args.iter().map(|ret| rewrite_rvalue(ret,right)).collect::<Vec<_>>();
    Ok(out)
}
