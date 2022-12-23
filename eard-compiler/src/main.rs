mod config;

use std::{process::exit, fs::File, io::Write};

use eard_compiler_lib::{compiler::EardCompiler, compilation::EardCompilation, serialise::EardSerializeCode };
use config::Config;
use clap::Parser;

fn do_it(config: &Config) -> Result<(),String> {
    let mut compiler = EardCompiler::new()?;
    if config.optimise {
        compiler.set_optimise(true);
    }
    let mut output = EardSerializeCode::new();
    for src in &config.source {
        let mut compilation = EardCompilation::new(&compiler)?;
        let code = compilation.compile(src)?;
        output.add(code);
    }
    let binary = output.serialize()?;
    let mut output = File::create(&config.outfile).map_err(|e| format!("cannot write file: {}",e))?;
    output.write_all(&binary).map_err(|e| format!("cannot write file: {}",e))?;
    Ok(())
}

fn main() {
    match do_it(&Config::parse()) {
        Ok(()) => { exit(0); }
        Err(e) => {
            eprintln!("{}",e);
            exit(1);
        }
    }
}
