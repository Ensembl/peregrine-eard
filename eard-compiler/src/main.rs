mod config;

use std::{process::exit, fs::File, io::{Write, self}};

use eard_compiler_lib::{compiler::EardCompiler, compilation::EardCompilation, serialise::EardSerializeCode };
use config::Config;
use clap::Parser;

fn do_it(config: &Config) -> Result<(),String> {
    let mut compiler = EardCompiler::new()?;
    if config.optimise {
        compiler.set_optimise(true);
    }
    if let Some(v) = config.bytecode {
        compiler.set_target_version(v);
    }
    if config.verbose {
        compiler.set_verbose(true);
    }
    let mut output = EardSerializeCode::new();
    for src in &config.source {
        let mut compilation = EardCompilation::new(&compiler)?;
        let code = compilation.compile(src)?;
        output.add(code);
    }
    let binary = match &config.format {
        config::Format::Standard => {
            output.serialize()?
        },
        config::Format::Expanded => {
            output.serialize_json().as_bytes().to_vec()
        }
    };
    if config.outfile == "-" {
        let mut f = io::stdout().lock();
        f.write_all(&binary).map_err(|e| format!("cannot write file: {}",e))?;
    } else {
        let mut f = File::create(&config.outfile).map_err(|e| format!("cannot write file: {}",e))?;
        f.write_all(&binary).map_err(|e| format!("cannot write file: {}",e))?;
    }
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
