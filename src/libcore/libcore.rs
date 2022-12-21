use ordered_float::OrderedFloat;
use crate::{compiler::EarpCompiler, model::{FullConstant, Constant}, source::FixedSourceSource};

use super::foldseq::{fold_bound, fold_total, fold_length, fold_push, fold_finseq, fold_infseq};

fn fold_add(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a+*b))])
    } else {
        None
    }
}

fn fold_sub(inputs: &[Option<FullConstant>]) -> Option<Vec<FullConstant>> {
    if let (Some(Some(FullConstant::Atomic(Constant::Number(a)))),
            Some(Some(FullConstant::Atomic(Constant::Number(b))))) = 
                (inputs.get(0),inputs.get(1)) {
        Some(vec![FullConstant::Atomic(Constant::Number(*a-*b))])
    } else {
        None
    }
}

pub(crate) fn libcore_add(compiler: &mut EarpCompiler) -> Result<(),String> {
    compiler.add_constant_folder("libcore__infseq",fold_infseq)?;
    compiler.add_constant_folder("libcore__finseq",fold_finseq)?;
    compiler.add_constant_folder("libcore__push",fold_push)?;
    compiler.add_constant_folder("libcore__length",fold_length)?;
    compiler.add_constant_folder("libcore__total",fold_total)?;
    compiler.add_constant_folder("libcore__bound",fold_bound)?;
    compiler.add_constant_folder("libcore__add",fold_add)?;
    compiler.add_constant_folder("libcore__sub",fold_sub)?;
    Ok(())
}

pub(crate) fn libcore_sources() -> FixedSourceSource {
    FixedSourceSource::new_vec(vec![
        ("libcore",include_str!("earp/libcore.earp")),
        ("sequences",include_str!("earp/sequences.earp")),
        ("maths",include_str!("earp/maths.earp")),
        ("print",include_str!("earp/print.earp"))
    ])
}
