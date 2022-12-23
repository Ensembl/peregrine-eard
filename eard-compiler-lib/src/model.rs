use std::{fmt::{self, Display}, cmp::Ordering, collections::HashMap, convert::Infallible};
use minicbor::{encode::{Error}, Encoder};
use ordered_float::OrderedFloat;
use crate::source::ParsePosition;

pub(crate) fn sepfmt<X>(input: &mut dyn Iterator<Item=X>, sep: &str, prefix: &str) -> String where X: fmt::Debug {
    input.map(|x| format!("{}{:?}",prefix,x)).collect::<Vec<_>>().join(sep)
}

#[derive(PartialEq,PartialOrd,Eq,Ord,Clone)]
pub enum Constant {
    Number(OrderedFloat<f64>),
    String(String),
    Boolean(bool)
}

impl Constant {
    pub(crate) fn to_atomic_type(&self) -> AtomicTypeSpec {
        match self {
            Constant::Number(_) => AtomicTypeSpec::Number,
            Constant::String(_) => AtomicTypeSpec::String,
            Constant::Boolean(_) => AtomicTypeSpec::Boolean
        }
    }
}

impl Constant {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        match self {
            Constant::Number(n) => { encoder.f64(n.0)?; },
            Constant::String(s) => { encoder.str(s)?; },
            Constant::Boolean(b) => { encoder.bool(*b)?; }
        }
        Ok(())
    }
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",match self {
            Constant::Number(n) => n.to_string(),
            Constant::String(s) => format!("{:?}",s),
            Constant::Boolean(b) => {
                (if *b { "true" } else { "false" }).to_string()
            }
        })
    }
}

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone)]
pub enum FullConstant {
    Atomic(Constant),
    Finite(Vec<Constant>),
    Infinite(Constant)
}

impl FullConstant {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        match self {
            FullConstant::Atomic(c) => {
                c.encode(encoder)?;
            },
            FullConstant::Finite(seq) => {
                encoder.array(seq.len() as u64)?;
                for c in seq {
                    c.encode(encoder)?;
                }
            },
            FullConstant::Infinite(c) => {
                encoder.begin_map()?.str("")?;
                c.encode(encoder)?;
                encoder.end()?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for FullConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(a) => write!(f,"{:?}",a),
            Self::Finite(s) => write!(f,"[{}]",sepfmt(&mut s.iter(),",","")),
            Self::Infinite(a) => write!(f,"[{:?},...]",a),
        }
    }
}

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

#[cfg(test)]
struct AllocDumper {
    next_call: usize,
    seen: HashMap<usize,usize>
}

#[cfg(test)]
impl AllocDumper {
    fn new() -> AllocDumper {
        AllocDumper { next_call: 0, seen: HashMap::new() }
    }

    fn get(&mut self, input: usize) -> usize {
        let (seen,next_call) = (&mut self.seen,&mut self.next_call);
        *seen.entry(input).or_insert_with(|| {
            *next_call +=1;
            *next_call
        })
    }
}

#[derive(Clone)]
pub struct Operation {
    pub(crate) position: ParsePosition,
    pub(crate) value: OperationValue
}

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

pub enum Step {
    Constant(usize,FullConstant),
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

#[derive(Clone)]
pub struct Metadata {
    pub(crate) group: String,
    pub(crate) name: String,
    pub(crate) version: u32
}

impl Metadata {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_array()?.str(&self.group)?.str(&self.name)?.u32(self.version)?.end()?;
        Ok(())
    }
}

pub struct CompiledBlock {
    pub constants: Vec<FullConstant>,
    pub program: Vec<(usize,Vec<usize>)>
}

impl CompiledBlock {
    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_map()?.str("constants")?.begin_array()?;
        for c in &self.constants {
            c.encode(encoder)?;
        }
        encoder.end()?.str("program")?.begin_array()?;
        for (opcode,opargs) in &self.program {
            encoder.array((opargs.len()+1) as u64)?.u32(*opcode as u32)?;
            for oparg in opargs {
                encoder.u32(*oparg as u32)?;
            }
        }
        encoder.end()?.end()?; /* program entry; main map */
        Ok(())
    }
}

impl fmt::Debug for CompiledBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"  constants:\n{}\n  program:\n{}\n",
            sepfmt(&mut self.constants.iter(),"\n","    "),
            sepfmt(&mut self.program.iter(),"\n","    ")
        )
    }
}

pub struct CompiledCode {
    pub metadata: Metadata,
    pub code: HashMap<String,CompiledBlock>
}

impl CompiledCode {
    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),Error<Infallible>> {
        encoder.begin_map()?.str("metadata")?;
        self.metadata.encode(encoder)?;
        encoder.str("blocks")?.begin_map()?;
        for (name,block) in self.code.iter() {
            encoder.str(name)?;
            block.encode(encoder)?;
        }
        encoder.end()?.end()?;
        Ok(())
    }
}

impl fmt::Debug for CompiledCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys = self.code.keys().collect::<Vec<_>>();
        keys.sort();
        for block in keys.iter() {
            let code = self.code.get(*block).unwrap();
            write!(f,"block: {}\n{:?}\n",block,code)?;
        }
        Ok(())
    }
}

#[derive(Clone,PartialEq,Eq)]
pub enum CodeModifier {
    World,
    Fold(String),
    Special(String)
}

impl fmt::Debug for CodeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeModifier::World => write!(f,"world")?,
            CodeModifier::Fold(s) => write!(f,"fold({})",s)?,
            CodeModifier::Special(s) => write!(f,"special({})",s)?
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeArgument {
    pub arg_type: TypeSpec,
    pub checks: Vec<Check>
}

impl fmt::Debug for CodeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?} {}",
            self.arg_type,
            sepfmt(&mut self.checks.iter()," ","")
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeImplVariable {
    pub reg_id: usize,
    pub arg_type: TypeSpec
}

impl fmt::Debug for CodeImplVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"r{} : {:?}",
            self.reg_id,
            self.arg_type
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum CodeImplArgument {
    Register(CodeImplVariable),
    Constant(Constant)
}

impl fmt::Debug for CodeImplArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(r) => write!(f,"{:?}",r),
            Self::Constant(c) => write!(f,"{:?}",c),
        }
    }
}

#[derive(Clone)]
pub enum CodeReturn {
    Register(CodeImplVariable),
    Repeat(usize)
}

impl fmt::Debug for CodeReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(r) => write!(f,"{:?}",r),
            Self::Repeat(r) => write!(f,"r{}",*r),
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum FuncProcModifier {
    Export,
    Entry,
    Version(Vec<String>)
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Variable {
    pub prefix: Option<String>,
    pub name: String
}

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f,"{}.{}",prefix,self.name)
        } else {
            write!(f,"{}",self.name)
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}",self)
    }
}

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
            AtomicTypeSpec::Handle(s) => (3,s),
        }
    }
}

impl fmt::Debug for AtomicTypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number => write!(f,"number"),
            Self::String => write!(f,"string"),
            Self::Boolean => write!(f,"boolean"),
            Self::Handle(h) => write!(f,"handle({})",h),
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

#[derive(Clone)]
pub enum TypeRestriction {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    AnySequence
}

impl fmt::Debug for TypeRestriction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(v) => write!(f,"{:?}",v),
            Self::Sequence(v) => write!(f,"seq({:?})",v),
            Self::AnySequence => write!(f,"seq"),
        }
    }
}

#[derive(Clone)]
pub enum TypeSpec {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    Wildcard(String),
    SequenceWildcard(String)
}

impl TypeSpec {
    pub(crate) fn as_restriction(&self) -> Option<TypeRestriction> {
        match self {
            TypeSpec::Atomic(a) => Some(TypeRestriction::Atomic(a.clone())),
            TypeSpec::Sequence(s) => Some(TypeRestriction::Sequence(s.clone())),
            TypeSpec::Wildcard(_) => { None }
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

#[derive(Clone)]
pub struct Opcode(pub usize,pub Vec<usize>);

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"opcode {}{}{};",
                self.0,
                if self.1.len() > 0 { ", " } else { "" },
                sepfmt(&mut self.1.iter(),", ","r")
        )
    }
}

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
