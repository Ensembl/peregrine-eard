use crate::{controller::{compiler::EardCompiler, source::FixedSourceSource}};
use super::{foldseq::{fold_bound, fold_total, fold_length, fold_push, fold_finseq, fold_infseq, fold_if, fold_repeat, fold_index, fold_count, fold_enumerate }, foldmaths::{fold_add, fold_sub, fold_mul, fold_div, fold_gt, fold_ge, fold_not, fold_eq, fold_minus, fold_and, fold_or, fold_any, fold_all, fold_position, fold_mod, fold_max, fold_min_seq, fold_max_seq, fold_min}, foldstring::{fold_push_str, fold_split, fold_template, fold_join, fold_format, fold_strlen}, foldconvert::{fold_to_boolean, fold_to_number, fold_to_string}};

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
    compiler.add_constant_folder("libcore__mod",fold_mod)?;
    compiler.add_constant_folder("libcore__gt",fold_gt)?;
    compiler.add_constant_folder("libcore__ge",fold_ge)?;
    compiler.add_constant_folder("libcore__not",fold_not)?;
    compiler.add_constant_folder("libcore__eq",fold_eq)?;
    compiler.add_constant_folder("libcore__minus",fold_minus)?;
    compiler.add_constant_folder("libcore__and",fold_and)?;
    compiler.add_constant_folder("libcore__or",fold_or)?;
    compiler.add_constant_folder("libcore__if",fold_if)?;
    compiler.add_constant_folder("libcore__repeat",fold_repeat)?;
    compiler.add_constant_folder("libcore__index",fold_index)?;
    compiler.add_constant_folder("libcore__count",fold_count)?;
    compiler.add_constant_folder("libcore__enumerate",fold_enumerate)?;
    compiler.add_constant_folder("libcore__join",fold_join)?;
    compiler.add_constant_folder("libcore__push_str",fold_push_str)?;
    compiler.add_constant_folder("libcore__split",fold_split)?;
    compiler.add_constant_folder("libcore__template",fold_template)?;
    compiler.add_constant_folder("libcore__format",fold_format)?;
    compiler.add_constant_folder("libcore__any",fold_any)?;
    compiler.add_constant_folder("libcore__all",fold_all)?;
    compiler.add_constant_folder("libcore__position",fold_position)?;
    compiler.add_constant_folder("libcore__to_boolean",fold_to_boolean)?;
    compiler.add_constant_folder("libcore__to_number",fold_to_number)?;
    compiler.add_constant_folder("libcore__to_string",fold_to_string)?;
    compiler.add_constant_folder("libcore__max",fold_max)?;
    compiler.add_constant_folder("libcore__min",fold_min)?;
    compiler.add_constant_folder("libcore__max_seq",fold_max_seq)?;
    compiler.add_constant_folder("libcore__min_seq",fold_min_seq)?;
    compiler.add_constant_folder("libcore__strlen",fold_strlen)?;
    Ok(())
}

pub(crate) fn libcore_sources() -> FixedSourceSource {
    FixedSourceSource::new_vec(vec![
        ("libcore",include_str!("eard/libcore.eard")),
        ("sequences",include_str!("eard/sequences.eard")),
        ("print",include_str!("eard/print.eard")),
        ("eq",include_str!("eard/eq.eard")),
        ("ineq",include_str!("eard/ineq.eard")),
        ("logic",include_str!("eard/logic.eard")),
        ("arith",include_str!("eard/arith.eard")),
        ("cond",include_str!("eard/cond.eard")),
        ("string",include_str!("eard/string.eard")),
        ("convert",include_str!("eard/convert.eard")),
        ("bio",include_str!("eard/bio.eard")),
    ])
}
