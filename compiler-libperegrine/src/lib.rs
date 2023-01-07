mod colour;
mod style;

use colour::colour_macro;
use eard_compiler_lib::{EardCompiler, FixedSourceSource};
use style::style_macro;

pub fn libperegrine_add(compiler: &mut EardCompiler) -> Result<(),String> {
    compiler.add_expression_macro("colour",colour_macro)?;
    compiler.add_block_macro("style",style_macro)?;
    compiler.add_source(FixedSourceSource::new_vec(vec![
        ("libperegrine",include_str!("eard/libperegrine.eard")),
    ]));
    Ok(())
}
