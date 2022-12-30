mod controller {
    pub(crate) mod context;
    pub(crate) mod globalcontext;
    pub(crate) mod handles;
    pub(crate) mod interpreter;
    pub(crate) mod operation;
    pub(crate) mod program;
    pub(crate) mod value;
    pub(crate) mod objectcode;    
}

mod libcore {
    pub(crate) mod libcore;
    mod arith;
    mod checks;
    mod print;
    mod seqctors;
}

#[cfg(test)]
mod test {
    mod test;
}

pub use controller::interpreter::Interpreter;
pub use libcore::libcore::{ prepare_libcore, build_libcore, LibcoreBuilder };
