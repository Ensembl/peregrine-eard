#[derive(Debug,Clone)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool)
}

#[derive(Debug,Clone)]
pub enum CodeModifier {
    World
}

#[derive(Debug,Clone)]
pub struct CodeRegisterArgument {
    pub reg_id: usize,
    pub arg_types: Vec<TypeSpec>,
    pub checks: Vec<Check>
}

#[derive(Debug,Clone)]
pub enum CodeArgument {
    Register(CodeRegisterArgument),
    Constant(Constant)
}

#[derive(Debug,Clone)]
pub enum CodeReturn {
    Register(CodeRegisterArgument),
    Repeat(usize)
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum FuncProcModifier {
    Export
}

#[derive(Debug,Clone)]
pub struct Variable {
    pub prefix: Option<String>,
    pub name: String
}


#[derive(Debug,Clone)]
pub enum CheckType {
    Length,
    LengthOrInfinite,
    Reference,
    Sum
}

#[derive(Debug,Clone)]
pub struct Check {
    pub check_type: CheckType,
    pub name: String
}

#[derive(Debug,Clone)]
pub enum OrBundle<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String)
}

impl<T: std::fmt::Debug+Clone> OrBundle<T> {
    pub(crate) fn as_bundle(&self) -> Option<String> {
        match self {
            OrBundle::Normal(_) => None,
            OrBundle::Bundle(b) => Some(b.to_string())
        }
    }
}

#[derive(Debug,Clone)]
pub enum OrRepeater<T: std::fmt::Debug+Clone> {
    Normal(T),
    Repeater(String)
}

#[derive(Debug,Clone)]
pub enum OrBundleRepeater<T: std::fmt::Debug+Clone> {
    Normal(T),
    Bundle(String),
    Repeater(String)
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
}

#[derive(Debug,Clone)]
pub enum AtomicTypeSpec {
    Number,
    String,
    Boolean,
    Handle(String)
}

#[derive(Debug,Clone)]
pub enum TypeSpec {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    Wildcard(String),
    SequenceWildcard(String)
}

#[derive(Debug,Clone)]
pub struct ArgTypeSpec {
    pub arg_types: Vec<TypeSpec>,
    pub checks: Vec<Check>
}

#[derive(Debug,Clone)]
pub struct TypedArgument {
    pub id: String,
    pub typespec: ArgTypeSpec
}

#[derive(Debug,Clone)]
pub enum CodeCommand {
    Opcode(usize,Vec<usize>),
    Register(usize)
}

#[derive(Debug,Clone)]
pub struct CodeBlock {
    pub name: String,
    pub arguments: Vec<CodeArgument>,
    pub results: Vec<CodeReturn>,
    pub commands: Vec<CodeCommand>,
    pub modifiers: Vec<CodeModifier>
}
