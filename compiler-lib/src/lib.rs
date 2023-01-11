mod controller {
    pub(crate) mod compiler;
    pub(crate) mod compilation;
    pub(crate) mod source;
    pub(crate) mod compiled;
    pub(crate) mod serialise;    
}

mod frontend {
    pub(crate) mod buildtree;
    mod buildtreebuilder;    
    pub(crate) mod preprocess;
    pub(crate) mod parser;
    pub(crate) mod parsetree;   
    pub(crate) mod femodel; 
}

mod libcore {
    pub(crate) mod libcore;
    mod util;
    mod foldstring;
    mod foldseq;
    mod foldmaths;
    mod foldconvert;
}

mod middleend {
    pub(crate) mod possible;
    pub(crate) mod reduce;
    pub(crate) mod broadtyping;
    pub(crate) mod narrowtyping;
    pub(crate) mod checking;  
    pub(crate) mod culdesac;
    pub(crate) mod constfold;
    pub(crate) mod reuse;
    pub(crate) mod reorder;
    pub(crate) mod generate;
    pub(crate) mod spill;
    pub(crate) mod large;
}

mod model {
    pub(crate) mod checkstypes;
    pub(crate) mod codeblocks;
    pub(crate) mod compiled;
    pub(crate) mod constants;
    pub(crate) mod linear;
    pub(crate) mod operation;
    pub(crate) mod step;
}

mod test {
    #[cfg(test)]
    mod test;
    pub(crate) mod testutil;    
    #[cfg(test)]
    mod testharness;
}

mod unbundle {
    pub(crate) mod buildunbundle;
    mod unbundleaux;
    mod repeater;
    pub(crate) mod linearize;
}

mod util {
    pub(crate) mod toposort;
    pub(crate) mod equiv;
}

pub use crate::model::{
    compiled::ProgramName
};

pub use crate::controller::{
    compiler::EardCompiler, 
    compilation::EardCompilation, 
    serialise::EardSerializeCode,
    source::FixedSourceSource
};

/* these are all exported to allow macros in external libraries */
pub use crate::controller::source::ParsePosition;
pub use crate::frontend::femodel::{OrBundle, OrBundleRepeater };
pub use crate::frontend::parsetree::{ PTExpression, PTStatement, PTStatementValue, PTCall };
pub use crate::frontend::buildtree::{ Variable };
pub use crate::frontend::parser::parse_string;
pub use crate::model::constants::Constant;