use std::{process::exit, pin::Pin, future::Future, fs::{self}};
use async_std::task::block_on;
use clap::{Parser};
use eard_interp::{RunContext, LibcoreTemplate, build_libcore, InterpreterBuilder, Interpreter, prepare_libcore, Metadata};

#[derive(Parser, Debug)]
#[command(name = "eard cli interpreter")]
#[command(author = "Ensembl Webteam <ensembl-webteam@ebi.ac.uk>")]
#[command(version = "0.0")]
#[command(about = "Command line tool to run cli-only eard binaries", long_about = None)]
pub(crate) struct Config {
    /// Block to run
    #[arg(short = 'b', long)]
    pub(crate) block: Option<String>,

    /// Program to run
    #[arg(short = 'p', long)]
    pub(crate) program: Option<String>,    

    /// Source files to run
    pub(crate) source: String,
}

async fn call_up_async() -> Result<(),String> {
    Ok(())
}

struct LibcoreCli;

impl LibcoreTemplate for LibcoreCli {
    fn print(&self, s: &str) {
        println!("{}",s);
    }

    fn call_up(&self) -> Pin<Box<dyn Future<Output=Result<(),String>>>> {
        Box::pin(call_up_async())
    }
}

fn guess_block(interp: &Interpreter, program: &Metadata) -> Result<String,String> {
    let blocks = interp.list_blocks(&program);
    if blocks.contains(&"main".to_string()) { 
        Ok("main".to_string())
    } else if let Some(b) = blocks.first() {
        Ok(b.to_string())
    } else {
        Err(format!("no such program in file"))
    }
}

fn do_it(config: &Config) -> Result<(),String> {
    eprintln!("running {} ; program {} ; block {}",
        config.source,
        config.program.as_ref().map(|x| x.as_str()).unwrap_or("*any*"),
        config.block.as_ref().map(|x| x.as_str()).unwrap_or("*any*")
    );
    /* prepare an interpreter */
    let libcore_context = LibcoreCli;
    let mut builder = InterpreterBuilder::new();
    let libcore_builder = build_libcore(&mut builder)?;
    let mut interp = Interpreter::new(builder);
    /* read the source */
    let contents = fs::read(&config.source).map_err(|e| format!("cannot read {}: {}",config.source,e))?;
    /* add the source */
    interp.load(&contents)?;
    /* find the program */
    let programs = interp.list_programs();
    let first = programs.first().ok_or_else(|| format!("File contained no programs!"))?;
    let program = if let Some(program) = &config.program {
        let parts = program.split(":").collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(format!("program spec needs three, colon-separated parts"));
        }
        let version = if let Ok(v) = parts[2].parse::<u32>() { v } else {
            return Err(format!("version must be a positive integer"));
        };
        Metadata::new(&parts[0],&parts[1],version)
    } else {
        first.clone()
    };
    let block = config.block.as_ref()
        .map(|x| Ok(x.clone()))
        .unwrap_or_else(|| { guess_block(&interp,&program) })?;
    /* prepare a run */
    let mut context = RunContext::new();
    prepare_libcore(&mut context,&libcore_builder,libcore_context);
    /* run */
    block_on(interp.run(&program,&block,context))?;
    Ok(())
}

pub fn main() {
    match do_it(&Config::parse()) {
        Ok(()) => { exit(0); }
        Err(e) => {
            eprintln!("{}",e);
            exit(1);
        }
    } 
}
