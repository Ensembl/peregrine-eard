use css_color_parser::Color;
use eard_compiler_lib::{OrBundleRepeater, PTExpression, PTCall, Constant};
use ordered_float::OrderedFloat;

pub(crate) fn colour_macro(expr: &[OrBundleRepeater<PTExpression>], ctx: usize) -> Result<PTExpression, String> {
    if expr.len() != 1 {
        return Err(format!("constant macro expects single argument"));
    }
    let value = match &expr[0] {
        OrBundleRepeater::Normal(PTExpression::Constant(Constant::String(s))) => s,
        _ => { return Err(format!("constant macro expects single, string argument")); }
    };
    let colour = value.parse::<Color>().map_err(|e| e.to_string())?;
    Ok(PTExpression::Call(PTCall{
        name: "colour".to_string(),
        args: vec![
            OrBundleRepeater::Normal(PTExpression::Constant(Constant::Number(OrderedFloat(colour.r as f64)))),
            OrBundleRepeater::Normal(PTExpression::Constant(Constant::Number(OrderedFloat(colour.g as f64)))),
            OrBundleRepeater::Normal(PTExpression::Constant(Constant::Number(OrderedFloat(colour.b as f64)))),
            OrBundleRepeater::Normal(PTExpression::Constant(Constant::Number(OrderedFloat(colour.a as f64 * 255.)))),
        ],
        is_macro: false
    }))
}
