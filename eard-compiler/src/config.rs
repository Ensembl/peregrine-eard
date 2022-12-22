use clap::{Parser, ValueEnum};

#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,ValueEnum)]
pub(crate) enum Format {
    /// Standard format (binary)
    Standard,
    /// Expanded (for debugging)
    Expanded
}


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "eard compiler")]
#[command(author = "Ensembl Webteam <ensembl-webteam@ebi.ac.uk>")]
#[command(version = "0.0")]
#[command(about = "Compiles eard source into eard binaries", long_about = None)]
pub(crate) struct Config {
   /// Source files to compile
   #[arg(short, long)]
   pub(crate) source: Vec<String>,

   /// Output filename
   #[arg(short, long, default_value = "out.eard")]
   pub(crate) outfile: String,

   /// Target bytecode version
   #[arg(short, long)]
   pub(crate) bytecode: Option<u32>,

   /// Optimise
   #[arg(short = 'O', long, default_value_t = false)]
   pub(crate) optimise: bool,

   /// Format
   #[arg(short, long, value_enum, default_value_t = Format::Standard)]
   pub(crate) format: Format
}