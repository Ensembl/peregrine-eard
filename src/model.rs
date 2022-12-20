use std::{fmt::{self, Display}, sync::Arc, cmp::Ordering};
use ordered_float::OrderedFloat;

pub(crate) fn sepfmt<X>(input: &mut dyn Iterator<Item=X>, sep: &str, prefix: &str) -> String where X: fmt::Debug {
    input.map(|x| format!("{}{:?}",prefix,x)).collect::<Vec<_>>().join(sep)

}

#[derive(Clone)]
pub(crate) struct FilePosition {
    pub filename: String,
    pub line_no: u32
}

impl FilePosition {
    pub(crate) fn anon() -> FilePosition {
        FilePosition { filename: "*anon*".to_string(), line_no: 0 }
    }

    pub(crate) fn new(filename: &str) -> FilePosition {
        FilePosition { filename: filename.to_string(), line_no: 0 }
    }
}

impl fmt::Debug for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}:{}",self.filename,self.line_no)
    }
}

#[derive(Clone)]
struct PositionNode(Option<Arc<PositionNode>>,FilePosition);

impl PositionNode {
    fn to_str(&self, prefix: &str, suffix: &str) -> String {
        let rest = self.0.as_ref().map(|parent| {
            format!("{}",parent.to_str(prefix,suffix))
        }).unwrap_or("".to_string());
        format!("{}{:?}{}{}",prefix,self.1,suffix,rest)
    }

    pub(crate) fn contains(&self, filename: &str) -> bool {
        if filename == self.1.filename { return true; }
        self.0.as_ref().map(|p| p.contains(filename)).unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct ParsePosition(PositionNode,Arc<String>);

impl ParsePosition {
    pub(crate) fn new(filename: &str, variety: &str) -> ParsePosition {
        ParsePosition(PositionNode(None,FilePosition::new(filename)),Arc::new(variety.to_string()))
    }

    pub(crate) fn contains(&self, filename: &str) -> bool {
        self.0.contains(filename)
    }

    pub(crate) fn empty(variety: &str) -> ParsePosition {
        ParsePosition(PositionNode(None,FilePosition::anon()),Arc::new(variety.to_string()))
    }

    pub(crate) fn at_line(&self, line_no: u32) -> ParsePosition {
        let mut out = self.clone();
        (out.0).1.line_no = line_no;
        out
    }

    pub(crate) fn update(&mut self, file: &FilePosition) {
        let parent = (self.0).0.clone();
        *self = ParsePosition(PositionNode(parent,file.clone()),self.1.clone());
    }

    pub(crate) fn push(&self, file: &FilePosition) -> ParsePosition {
        ParsePosition(PositionNode(Some(Arc::new(self.0.clone())),file.clone()),self.1.clone())
    }

    pub(crate) fn last(&self) -> &FilePosition { &(self.0).1 }

    pub(crate) fn last_str(&self) -> String { format!("{:?}",self.last()) }

    pub(crate) fn full_str(&self) -> String {
        let rest = (self.0).0.as_ref().map(|x|
            x.to_str(&format!(" ({} from ",self.1),")")
        ).unwrap_or("".to_string());
        format!("{:?}{}",(self.0).1,rest)
    }

    pub(crate) fn message(&self, msg: &str) -> String {
        format!("{} at {}",msg,self.full_str())
    }
}

impl fmt::Debug for ParsePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.full_str())
    }
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
    Code(usize,usize,Vec<usize>,Vec<usize>), // call,name,rets,args
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
                )
        }
    }
}

#[derive(Clone)]
pub struct Operation {
    pub(crate) position: ParsePosition,
    pub(crate) value: OperationValue
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {:?}",self.position.last_str(),self.value)
    }
}

pub enum Step {
    Constant(usize,FullConstant),
    Opcode(usize,Vec<usize>)
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::Constant(r,c) => write!(f,"r{} <- {:?}",r,c),
            Step::Opcode(opcode,args) => {
                write!(f,"opcode {}, {}",*opcode,sepfmt(&mut args.iter(),", ","r"))
            }
        }
    }
}

#[derive(Clone,PartialEq,Eq)]
pub enum CodeModifier {
    World,
    Fold(String)
}

impl fmt::Debug for CodeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeModifier::World => write!(f,"world")?,
            CodeModifier::Fold(s) => write!(f,"fold({})",s)?
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
    Export
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

#[derive(Clone)]
pub enum OrBundle<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String)
}

impl<T: fmt::Debug+Clone> fmt::Debug for OrBundle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(v) => write!(f,"{:?}",v),
            Self::Bundle(b) => write!(f,"*{}",b),
        }
    }
}

#[derive(Clone)]
pub enum OrBundleRepeater<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String),
    Repeater(String)
}

impl<T: fmt::Debug+Clone> fmt::Debug for OrBundleRepeater<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(v) => write!(f,"{:?}",v),
            Self::Bundle(b) => write!(f,"*{}",b),
            Self::Repeater(r) => write!(f,"**{}",r),
        }
    }
}

impl<T: std::fmt::Debug+Clone> OrBundleRepeater<T> {
    pub(crate) fn map<F,U: std::fmt::Debug+Clone>(&self, cb: F) -> OrBundleRepeater<U> where F: FnOnce(&T) -> U {
        match self {
            OrBundleRepeater::Normal(n) => OrBundleRepeater::Normal(cb(n)),
            OrBundleRepeater::Bundle(b) => OrBundleRepeater::Bundle(b.clone()),
            OrBundleRepeater::Repeater(r) => OrBundleRepeater::Repeater(r.clone())
        }
    }

    pub(crate) fn map_result<F,U: std::fmt::Debug+Clone,E>(&self, cb: F) -> Result<OrBundleRepeater<U>,E> where F: FnOnce(&T) -> Result<U,E> {
        Ok(match self {
            OrBundleRepeater::Normal(n) => OrBundleRepeater::Normal(cb(n)?),
            OrBundleRepeater::Bundle(b) => OrBundleRepeater::Bundle(b.clone()),
            OrBundleRepeater::Repeater(r) => OrBundleRepeater::Repeater(r.clone())
        })
    }

    pub(crate) fn is_repeater(&self) -> bool {
        match self {
            OrBundleRepeater::Repeater(_) => true,
            _ => false
        }
    }

    pub(crate) fn no_repeater(&self) -> Result<OrBundle<T>,String> {
        Ok(match self {
            OrBundleRepeater::Normal(n) => OrBundle::Normal(n.clone()),
            OrBundleRepeater::Bundle(b) => OrBundle::Bundle(b.clone()),
            OrBundleRepeater::Repeater(_) => { return Err(format!("unexpected repeater")) }
        })
    }

    pub(crate) fn skip_repeater(&self) -> Option<OrBundle<T>> {
        match self {
            OrBundleRepeater::Normal(n) => Some(OrBundle::Normal(n.clone())),
            OrBundleRepeater::Bundle(b) => Some(OrBundle::Bundle(b.clone())),
            OrBundleRepeater::Repeater(_) => { None }
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
    Check(usize,CheckType,usize, bool), // reg, type, index, force
    Constant(usize,Constant),
    Copy(usize,usize), // to,from
    Code(usize,usize,Vec<usize>,Vec<usize>), // name,call,index,rets,args
    Type(usize,Vec<TypeRestriction>),
    WildEquiv(Vec<usize>)
}

impl fmt::Debug for LinearStatementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check(v, ct, c,force) => {
                let force = if *force { "f" } else { "" };
                write!(f,"r{:?} <check>{} {:?} {:?}",v,force,ct,c)
            },
            Self::Type(v, c) => write!(f,"r{:?} <type> {:?}",v,c),
            Self::WildEquiv(r) => write!(f,"<wild-equiv> {}",sepfmt(&mut r.iter(),", ","r")),
            Self::Constant(v,c) => write!(f,"r{:?} <constant> {:?}",v,c),
            Self::Copy(to,from) => write!(f,"r{:?} <copy-from> r{:?}",*to,*from),
            Self::Code(call,name,rets,args) => {
                write!(f,"{} ({}#{}) {}",
                    sepfmt(&mut rets.iter()," ","r"),name,call,sepfmt(&mut args.iter()," ","r"))
            }
        }
    }
}

#[derive(Clone)]
pub struct LinearStatement {
    pub value: LinearStatementValue,
    pub position: ParsePosition
}

impl fmt::Debug for LinearStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{} {:?}",self.position.last_str(),self.value)
    }
}
