use crate::{controller::{compiler::EardCompiler, source::FixedSourceSource}};
use super::{foldseq::{fold_bound, fold_total, fold_length, fold_push, fold_finseq, fold_infseq }, foldmaths::{fold_add, fold_sub, fold_mul, fold_div, fold_gt, fold_ge, fold_not}};

pub(crate) fn libcore_add(compiler: &mut EardCompiler) -> Result<(),String> {
    compiler.add_constant_folder("libcore__infseq",fold_infseq)?;
    compiler.add_constant_folder("libcore__finseq",fold_finseq)?;
    compiler.add_constant_folder("libcore__push",fold_push)?;
    compiler.add_constant_folder("libcore__length",fold_length)?;
    compiler.add_constant_folder("libcore__total",fold_total)?;
    compiler.add_constant_folder("libcore__bound",fold_bound)?;
    compiler.add_constant_folder("libcore__add",fold_add)?;
    compiler.add_constant_folder("libcore__sub",fold_sub)?;
    compiler.add_constant_folder("libcore__mul",fold_mul)?;
    compiler.add_constant_folder("libcore__div",fold_div)?;
    compiler.add_constant_folder("libcore__gt",fold_gt)?;
    compiler.add_constant_folder("libcore__ge",fold_ge)?;
    compiler.add_constant_folder("libcore__not",fold_not)?;
    Ok(())
}

pub(crate) fn libcore_sources() -> FixedSourceSource {
    FixedSourceSource::new_vec(vec![
        ("libcore",include_str!("eard/libcore.eard")),
        ("sequences",include_str!("eard/sequences.eard")),
        ("maths",include_str!("eard/maths.eard")),
        ("print",include_str!("eard/print.eard"))
    ])
}
