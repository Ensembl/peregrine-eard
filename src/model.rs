use std::{fmt::{self, Display}, sync::Arc};

fn sepfmt<X>(input: &mut dyn Iterator<Item=X>, sep: &str, prefix: &str) -> String where X: fmt::Debug {
    input.map(|x| format!("{}{:?}",prefix,x)).collect::<Vec<_>>().join(sep)

}

#[derive(Clone)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool)
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

#[derive(Clone,PartialEq,Eq)]
pub enum CodeModifier {
    World
}

impl fmt::Debug for CodeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeModifier::World => write!(f,"world")?
        }
        Ok(())
    }
}

/*
#[derive(Clone)]
pub struct CodeRegisterArgument {
    pub reg_id: usize,
    pub arg_types: Vec<TypeSpec>,
    pub checks: Vec<Check>
}

impl fmt::Debug for CodeRegisterArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"r{}",self.reg_id)?;
        if self.arg_types.len() > 0 || self.checks.len() > 0 {
            write!(f," : {} {}",
                sepfmt(&mut self.arg_types.iter(),"|",""),
                sepfmt(&mut self.checks.iter()," ","")
            )?;
        }
        Ok(())
    }
}
*/

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

#[derive(Debug,Clone)]
pub enum CheckType {
    Length,
    LengthOrInfinite,
    Reference,
    Sum
}

#[derive(Clone)]
pub struct Check {
    pub check_type: CheckType,
    pub name: String
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

impl<T: std::fmt::Debug+Clone> OrBundle<T> {
    pub(crate) fn as_bundle(&self) -> Option<String> {
        match self {
            OrBundle::Normal(_) => None,
            OrBundle::Bundle(b) => Some(b.to_string())
        }
    }
}

#[derive(Clone)]
pub enum OrRepeater<T: std::fmt::Debug+Clone> {
    Normal(T),
    Repeater(String)
}

impl<T: fmt::Debug+Clone> fmt::Debug for OrRepeater<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(v) => write!(f,"{:?}",v),
            Self::Repeater(r) => write!(f,"**{}",r),
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

#[derive(Clone)]
pub enum AtomicTypeSpec {
    Number,
    String,
    Boolean,
    Handle(String)
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

#[derive(Clone)]
pub enum TypeSpec {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    Wildcard(String),
    SequenceWildcard(String)
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
pub enum CodeCommand {
    Opcode(usize,Vec<usize>),
    Register(usize)
}

impl fmt::Debug for CodeCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opcode(opcode,args) => write!(f,"opcode {}{}{};",
                *opcode,
                if args.len() > 0 { ", " } else { "" },
                sepfmt(&mut args.iter(),", ","r")
            ),
            Self::Register(r) => write!(f,"register r{};",*r)
        }
    }
}

#[derive(Clone)]
pub struct ImplBlock {
    pub arguments: Vec<CodeImplArgument>,
    pub results: Vec<CodeReturn>,
    pub commands: Vec<CodeCommand>,
}

impl fmt::Debug for ImplBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f," impl ({})",
            sepfmt(&mut self.arguments.iter(),", ","")
        )?;
        if self.results.len() > 0 {
            write!(f," -> ({}) ",
                sepfmt(&mut self.results.iter(),", ","")
            )?;
        }
        write!(f," {{\n{}\n}}\n\n",sepfmt(&mut self.commands.iter(),"\n","  "))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CodeBlock {
    pub name: String,
    pub arguments: Vec<CodeArgument>,
    pub results: Vec<CodeArgument>,
    pub impls: Vec<ImplBlock>,
    pub modifiers: Vec<CodeModifier>
}

impl fmt::Debug for CodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",sepfmt(&mut self.modifiers.iter()," ",""))?;
        write!(f," code {}({})",
            self.name,
            sepfmt(&mut self.arguments.iter(),", ","")
        )?;
        if self.results.len() > 0 {
            write!(f," -> ({}) ",
                sepfmt(&mut self.results.iter(),", ","")
            )?;
        }
        write!(f," {{\n{}\n}}\n\n",sepfmt(&mut self.impls.iter(),"\n","  "))?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum LinearStatementValue {
    Check(usize,Check),
    Constant(usize,Constant),
    Copy(usize,usize), // to,from
    Code(usize,Vec<usize>,Vec<usize>,bool), // name,rets,args
    Type(usize,Vec<TypeSpec>)
}

impl fmt::Debug for LinearStatementValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check(v, c) => write!(f,"r{:?} <check> {:?}",v,c),
            Self::Type(v, c) => write!(f,"r{:?} <type> {:?}",v,c),
            Self::Constant(v,c) => write!(f,"r{:?} <constant> {:?}",v,c),
            Self::Copy(to,from) => write!(f,"r{:?} <copy-from> r{:?}",*to,*from),
            Self::Code(name,rets,args,world) => {
                let world = if *world { "w" } else { "" };
                write!(f,"{} ({}){} {}",
                    sepfmt(&mut rets.iter()," ","r"),name,world,sepfmt(&mut args.iter()," ","r"))
            }
        }
    }
}

#[derive(Clone)]
pub struct LinearStatement {
    pub value: LinearStatementValue,
    pub file: Arc<Vec<String>>,
    pub line_no: usize
}

impl fmt::Debug for LinearStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.file.as_ref().last().map(|x| x.as_str()).unwrap_or("");
        write!(f,"{}:{} {:?}",file,self.line_no,self.value)
    }
}
