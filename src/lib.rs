mod frontend {
    pub(crate) mod buildtree;
    mod buildtreebuilder;    
    pub(crate) mod preprocess;
    pub(crate) mod parser;
    pub(crate) mod parsetree;    
}

mod unbundle {
    pub(crate) mod buildunbundle;
    mod unbundleaux;
    mod repeater;
    pub(crate) mod linearize;
}

mod checking;
mod codeblocks;
mod compiler;
mod equiv;
mod model;
mod reduce;
mod typing;

#[cfg(test)]
mod test;

#[cfg(test)]
mod testharness;
