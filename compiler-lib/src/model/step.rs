use std::fmt;
use crate::test::testutil::sepfmt;
use super::constants::OperationConstant;

pub(crate) enum Step {
    Constant(usize,OperationConstant),
    Opcode(usize,Vec<usize>),
    Entry(String)
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::Constant(r,c) => write!(f,"r{} <- {:?}",r,c),
            Step::Opcode(opcode,args) => {
                write!(f,"opcode {}, {}",*opcode,sepfmt(&mut args.iter(),", ","r"))
            },
            Step::Entry(s) => {
                write!(f,"entrypoint {}",s)
            }
        }
    }
}

