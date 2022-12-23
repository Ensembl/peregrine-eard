use std::fmt;

use super::{checkstypes::{CheckType, TypeRestriction}, constants::Constant};

#[derive(Clone)]
pub enum LinearStatementValue {
    Check(String,usize,CheckType,usize, bool), // source-name, reg, type, index, force
    Constant(usize,Constant),
    Copy(usize,usize), // to,from
    Code(usize,usize,Vec<usize>,Vec<usize>), // call,index,rets,args
    Type(usize,Vec<TypeRestriction>),
    WildEquiv(Vec<usize>),
    Entry(String)
}

#[cfg(test)]
use crate::test::testutil::AllocDumper;
use crate::{test::testutil::sepfmt, source::ParsePosition};

impl LinearStatementValue {
    #[cfg(test)]
    fn dump(&self, ad: &mut AllocDumper) -> String {
        match self {
            Self::Check(name,v, ct, c,force) => {
                let force = if *force { "f" } else { "" };
                format!("r{:?} <check:{}>{} {:?} {:?}",v,name,force,ct,c)
            },
            Self::Type(v, c) => format!("r{:?} <type> {:?}",v,c),
            Self::WildEquiv(r) => format!("<wild-equiv> {}",sepfmt(&mut r.iter(),", ","r")),
            Self::Constant(v,c) => format!("r{:?} <constant> {:?}",v,c),
            Self::Copy(to,from) => format!("r{:?} <copy-from> r{:?}",*to,*from),
            Self::Code(call,name,rets,args) => {
                format!("{} ({}#{}) {}",
                    sepfmt(&mut rets.iter()," ","r"),ad.get(*name),call,sepfmt(&mut args.iter()," ","r"))
            },
            Self::Entry(name) => format!("<entry> {:?}",name)
        }
    }
}

impl fmt::Debug for LinearStatementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check(name,v, ct, c,force) => {
                let force = if *force { "f" } else { "" };
                write!(f,"r{:?} <check:{}>{} {:?} {:?}",v,name,force,ct,c)
            },
            Self::Type(v, c) => write!(f,"r{:?} <type> {:?}",v,c),
            Self::WildEquiv(r) => write!(f,"<wild-equiv> {}",sepfmt(&mut r.iter(),", ","r")),
            Self::Constant(v,c) => write!(f,"r{:?} <constant> {:?}",v,c),
            Self::Copy(to,from) => write!(f,"r{:?} <copy-from> r{:?}",*to,*from),
            Self::Code(call,name,rets,args) => {
                write!(f,"{} ({}#{}) {}",
                    sepfmt(&mut rets.iter()," ","r"),name,call,sepfmt(&mut args.iter()," ","r"))
            },
            Self::Entry(name) => write!(f,"<entry> {:?}",name)
        }
    }
}

#[derive(Clone)]
pub struct LinearStatement {
    pub value: LinearStatementValue,
    pub position: ParsePosition
}

impl LinearStatement {
    #[cfg(test)]
    fn dump(&self, ad: &mut AllocDumper) -> String {
        format!("{} {}",self.position.last_str(),self.value.dump(ad))
    }
}

impl fmt::Debug for LinearStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {:?}",self.position.last_str(),self.value)
    }
}

#[cfg(test)]
pub(crate) fn dump_linear(linear: &[LinearStatement]) -> String {
    let mut ad = AllocDumper::new();
    let mut out = String::new();
    for stmt in linear {
        out.push_str(&stmt.dump(&mut ad));
        out.push('\n');
    }
    out
}
