mod frontend {
    pub(crate) mod buildtree;
    mod buildtreebuilder;    
    pub(crate) mod preprocess;
    pub(crate) mod parser;
    pub(crate) mod parsetree;    
}

mod libcore {
    pub(crate) mod libcore;
}

mod middleend {
    pub(crate) mod reduce;
    pub(crate) mod broadtyping;
    pub(crate) mod narrowtyping;
    pub(crate) mod checking;  
    pub(crate) mod culdesac;
    pub(crate) mod constfold;
}

mod unbundle {
    pub(crate) mod buildunbundle;
    mod unbundleaux;
    mod repeater;
    pub(crate) mod linearize;
}

mod codeblocks;
pub mod compiler;
pub mod compilation;
mod equiv;
mod model;
mod reuse;
pub(crate) mod toposort;
mod reorder;
mod generate;
mod spill;


#[cfg(test)]
mod test;

#[cfg(test)]
mod testharness;
