use crate::{model::OrBundleRepeater, buildtree::{BTProcCall, BTExpression, BTLValue}};

pub(crate) fn list_repeaters(args: &[OrBundleRepeater<BTExpression>]) -> Result<Vec<String>,String> {
    let mut repeaters = vec![];
    for arg in args {
        match arg {
            OrBundleRepeater::Normal(x) => {
                match x {
                    BTExpression::Function(f) => {
                        repeaters.append(&mut list_repeaters(&f.args)?);
                    },
                    _ => {},
                }
            },
            OrBundleRepeater::Bundle(_) => {},
            OrBundleRepeater::Repeater(r) => { repeaters.push(r.to_string()) }
        }
    }
    Ok(repeaters)
}

pub(crate) fn is_good_repeater(call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<bool,String> {
    /* A good repeater has a single use of the repeater glyph (**) on each side and has a
     * single lhs variable.
     */
    let rets = match &call.rets {
        Some(x) if x.len() == 1 => { &x[0] },
        _ => { return Ok(false); } // len != 1 on lhs
    };
    match rets {
        OrBundleRepeater::Repeater(_) => {},
        _ => { return Ok(false); } // lhs is not repeater
    }
    Ok(list_repeaters(&call.args)?.len() == 1)
}

pub(crate) fn uses_repeater(call: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<bool,String> {
    for ret in call.rets.as_ref().unwrap_or(&vec![]).iter() {
        match ret {
            OrBundleRepeater::Repeater(_) => { return Ok(true); },
            _ => {}
        }
    }
    Ok(list_repeaters(&call.args)?.len() > 0)
}
