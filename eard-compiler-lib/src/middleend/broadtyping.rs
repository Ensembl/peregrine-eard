use std::{collections::HashMap, fmt};

use crate::{model::{AtomicTypeSpec, LinearStatement, LinearStatementValue, TypeSpec, TypeRestriction}, frontend::{buildtree::{BuildTree, BTTopDefn}}, source::ParsePosition};

#[derive(Clone,PartialEq,Eq)]
pub(crate) enum BroadType {
    Atomic(AtomicTypeSpec),
    Sequence
}

impl BroadType {
    pub(crate) fn from_typespec(spec: &TypeSpec) -> Result<BroadType,String> {
        match spec {
            TypeSpec::Atomic(a) => Ok(BroadType::Atomic(a.clone())),
            TypeSpec::Sequence(_) => Ok(BroadType::Sequence),
            TypeSpec::Wildcard(w) => Err(w.to_string()),
            TypeSpec::SequenceWildcard(_) => Ok(BroadType::Sequence),
        }
    }

    pub(crate) fn from_restriction(spec: &TypeRestriction) -> BroadType {
        match spec {
            TypeRestriction::Atomic(a) => BroadType::Atomic(a.clone()),
            TypeRestriction::Sequence(_) => BroadType::Sequence,
            TypeRestriction::AnySequence => BroadType::Sequence
        }
    }
}

impl fmt::Debug for BroadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(t) => write!(f,"{:?}",t),
            Self::Sequence => write!(f, "seq"),
        }
    }
}

pub(crate) struct BroadTyping<'a> {
    bt: &'a BuildTree,
    position: ParsePosition,
    types: HashMap<usize,BroadType>,
    blocks: HashMap<usize,usize>
}

impl<'a> BroadTyping<'a> {
    fn new(bt: &'a BuildTree) -> BroadTyping<'a> {
        BroadTyping { bt, types: HashMap::new(), position: ParsePosition::empty("called"), blocks: HashMap::new() }
    }

    fn get(&self, reg: usize) -> BroadType {
        self.types.get(&reg).expect(&format!("register r{} used before set",reg)).clone()
    }

    fn add(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Constant(reg,c) => {
                self.types.insert(*reg,BroadType::Atomic(c.to_atomic_type()));
            },
            LinearStatementValue::Copy(dst,src) => {
                self.types.insert(*dst,self.get(*src));
            },
            LinearStatementValue::Code(call,index,dst,src) => {
                let defn = match self.bt.get_by_index(*index)? {
                    BTTopDefn::FuncProc(_) => { panic!("code index did not refer to code!") },
                    BTTopDefn::Code(defn) => defn
                };
                let src_types = src.iter().map(|r| self.get(*r)).collect::<Vec<_>>();
                let (dst_types,block_index) = defn.broad_typing(&src_types)?;
                if dst.len() != dst_types.len() {
                    return Err(format!("code did not return expected argument count"));
                }
                self.blocks.insert(*call,block_index);
                for (reg,broad) in dst.iter().zip(dst_types.iter()) {
                    self.types.insert(*reg,broad.clone());
                }
            },
            LinearStatementValue::Type(reg,spec) => {
                let got = self.get(*reg);
                for broad in spec {
                    let want = BroadType::from_restriction(broad);
                    if want != got {
                        return Err(format!("type check failed: expected {:?} got {:?}",want,got));
                    }
                }
            },
            LinearStatementValue::Check(_,_,_,_,_) => {},
            LinearStatementValue::WildEquiv(_) => {},
        }
        Ok(())
    }

    fn take(self) -> (HashMap<usize,BroadType>,HashMap<usize,usize>) { (self.types,self.blocks) }
}

pub(crate) fn broad_type(bt: &BuildTree, stmts: &[LinearStatement]) -> Result<(HashMap<usize,BroadType>,HashMap<usize,usize>),String> {
    let mut typing = BroadTyping::new(bt);
    for stmt in stmts {
        typing.add(stmt).map_err(|e| typing.position.message(&e))?;
    }
    Ok(typing.take())
}
