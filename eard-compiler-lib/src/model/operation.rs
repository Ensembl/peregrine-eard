use std::fmt;
use crate::{test::testutil::sepfmt, controller::source::ParsePosition};
use super::constants::FullConstant;

#[derive(Clone)]
pub enum OperationValue {
    Constant(usize,FullConstant),
    Code(usize,usize,Vec<usize>,Vec<usize>), // call,name,rets,args,
    Entry(String)
}

impl fmt::Debug for OperationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationValue::Constant(r,c) => write!(f,"r{} <- {:?}",r,c),
            OperationValue::Code(call,name,rets,args) => 
                write!(f,"{} ({}#{}) {}",
                    sepfmt(&mut rets.iter()," ","r"),
                    *name,*call,
                    sepfmt(&mut args.iter()," ","r")
                ),
            OperationValue::Entry(s) => {
                write!(f,"entrypoint {}",s)
            }
        }
    }
}

#[derive(Clone)]
pub struct Operation {
    pub(crate) position: ParsePosition,
    pub(crate) value: OperationValue
}

#[cfg(test)]
use crate::test::testutil::AllocDumper;

impl OperationValue {
    #[cfg(test)]
    fn dump(&self, ad: &mut AllocDumper) -> String {
        match self {
            OperationValue::Constant(r,c) => format!("r{} <- {:?}",r,c),
            OperationValue::Code(call,name,rets,args) => 
                format!("{} ({}#{}) {}",
                    sepfmt(&mut rets.iter()," ","r"),
                    ad.get(*name),*call,
                    sepfmt(&mut args.iter()," ","r")
                ),
            OperationValue::Entry(s) => {
                format!("entrypoint {}",s)
            }    
        }
    }
}

impl Operation {
    #[cfg(test)]
    fn dump(&self, ad: &mut AllocDumper) -> String {
        format!("{} {}",self.position.last_str(),self.value.dump(ad))
    }
}

#[cfg(test)]
pub(crate) fn dump_opers(opers: &[Operation]) -> String {
    let mut ad = AllocDumper::new();
    let mut out = String::new();
    for oper in opers {
        out.push_str(&oper.dump(&mut ad));
        out.push('\n');
    }
    out
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {:?}",self.position.last_str(),self.value)
    }
}
