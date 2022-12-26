use std::{fmt, cmp::Ordering, collections::HashSet};

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum CheckType {
    Length,
    LengthOrInfinite,
    Reference,
    Sum
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Check {
    pub check_type: CheckType,
    pub name: String,
    pub force: bool
}

impl fmt::Debug for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.check_type {
            CheckType::Length => write!(f,"length({})",self.name),
            CheckType::LengthOrInfinite => write!(f,"length({}...)",self.name),
            CheckType::Reference => write!(f,"ref({})",self.name),
            CheckType::Sum => write!(f,"total({})",self.name)
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum AtomicTypeSpec {
    Number,
    String,
    Boolean,
    Handle(String)
}

impl AtomicTypeSpec {
    fn ord_key(&self) -> (usize,&str) {
        match self {
            AtomicTypeSpec::Boolean => (0,""),
            AtomicTypeSpec::Number => (1,""),
            AtomicTypeSpec::String => (2,""),
            AtomicTypeSpec::Handle(s) => (3,s)
        }
    }
}

impl fmt::Debug for AtomicTypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number => write!(f,"number"),
            Self::String => write!(f,"string"),
            Self::Boolean => write!(f,"boolean"),
            Self::Handle(h) => write!(f,"handle({})",h)
        }
    }
}

impl PartialOrd for AtomicTypeSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AtomicTypeSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ord_key().cmp(&other.ord_key())
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub(crate) enum TypeRestriction {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    AnySequence,
    AnyAtomic
}

impl fmt::Debug for TypeRestriction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(v) => write!(f,"{:?}",v),
            Self::Sequence(v) => write!(f,"seq({:?})",v),
            Self::AnySequence => write!(f,"seq"),
            Self::AnyAtomic => write!(f,"atom"),
        }
    }
}

fn add_sequences(output: &mut HashSet<TypeRestriction>, input: &HashSet<TypeRestriction>) {
    output.extend(input.iter().filter(|r| {
        match r {
            TypeRestriction::Sequence(_) => true,
            _ => false
        }
    }).cloned())
}

pub(crate) fn intersect_restrictions(a: &HashSet<TypeRestriction>, b: &HashSet<TypeRestriction>) -> HashSet<TypeRestriction> {
    let mut out = HashSet::new();
    let a_any = a.contains(&TypeRestriction::AnySequence);
    let b_any = b.contains(&TypeRestriction::AnySequence);
    if a_any && b_any { out.insert(TypeRestriction::AnySequence); }
    if a_any { add_sequences(&mut out,b); }
    if b_any { add_sequences(&mut out,a); }
    out.extend(a.intersection(b).cloned());
    out
}

#[derive(Clone)]
pub enum TypeSpec {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    Wildcard(String),
    AtomWildcard(String),
    SequenceWildcard(String)
}

impl TypeSpec {
    pub(crate) fn as_restriction(&self) -> Option<TypeRestriction> {
        match self {
            TypeSpec::Atomic(a) => Some(TypeRestriction::Atomic(a.clone())),
            TypeSpec::Sequence(s) => Some(TypeRestriction::Sequence(s.clone())),
            TypeSpec::Wildcard(_) => { None },
            TypeSpec::AtomWildcard(_) => { Some(TypeRestriction::AnyAtomic) },
            TypeSpec::SequenceWildcard(_) =>  { Some(TypeRestriction::AnySequence) },
        }
    }
}

impl fmt::Debug for TypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(v) => write!(f,"{:?}",v),
            Self::Sequence(v) => write!(f,"seq({:?})",v),
            Self::Wildcard(v) => write!(f,"?{}",v),
            Self::SequenceWildcard(v) => write!(f,"seq(?{})",v),
            Self::AtomWildcard(v) => write!(f,"atom(?{})",v),
        }
    }
}

#[derive(Clone)]
pub struct ArgTypeSpec {
    pub arg_types: Vec<TypeSpec>,
    pub checks: Vec<Check>
}

impl fmt::Debug for ArgTypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i,arg) in self.arg_types.iter().enumerate() {
            write!(f,"{}{:?}",if i>0 {"|"} else{""},arg)?;
        }
        for check in &self.checks {
            write!(f," {:?}",check)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TypedArgument {
    pub id: String,
    pub typespec: ArgTypeSpec
}

impl fmt::Debug for TypedArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = self.typespec.arg_types.len() > 0 || self.typespec.checks.len() > 0;
        write!(f,"{}{}{:?}",self.id,if sep { ": "} else { "" },self.typespec)
    }
}