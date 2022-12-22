mod config;

use std::process::exit;

use eard_compiler_lib::{compiler::EardCompiler, compilation::EardCompilation};
use config::Config;
use clap::Parser;

fn do_it(config: &Config) -> Result<(),String> {
    let config = Config::parse();
    let compiler = EardCompiler::new()?;
    let compilation = EardCompilation::new(&compiler)?;
    eprintln!("{:?}",config);
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
