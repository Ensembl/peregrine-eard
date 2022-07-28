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
pub enum CallArg<E> {
    Expression(E),
    Bundle(String),
    Repeater(String)
}
