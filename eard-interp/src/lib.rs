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
    mod print;
}

#[cfg(test)]
mod test {
    mod test;
}
