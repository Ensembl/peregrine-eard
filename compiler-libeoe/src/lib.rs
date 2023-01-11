mod structmacro;
mod parser;
mod parsetree;

use eard_compiler_lib::{EardCompiler, FixedSourceSource};
use structmacro::struct_macro;

pub fn libeoe_add(compiler: &mut EardCompiler) -> Result<(),String> {
    compiler.add_block_macro("struct",struct_macro)?;
    compiler.add_source(FixedSourceSource::new_vec(vec![
        ("libeoe",include_str!("eard/libeoe.eard")),
    ]));
    Ok(())
}
