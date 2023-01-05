use eard_compiler_lib::{EardCompiler, FixedSourceSource};

pub fn libperegrine_add(compiler: &mut EardCompiler) -> Result<(),String> {
    compiler.add_source(FixedSourceSource::new_vec(vec![
        ("libperegrine",include_str!("eard/libperegrine.eard")),
    ]));
    Ok(())
}
