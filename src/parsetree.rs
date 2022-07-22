use std::sync::Arc;

use crate::{buildtree::{BuildTree, BTDefinition, BTCodeDefinition, BTDeclare, BuildContext}, model::{CodeModifier, Variable, Check, FuncProcModifier}};

pub(crate) fn at(msg: &str, pos: Option<(&[String],usize)>) -> String {
    if let Some((parents, line_no)) = pos {
        let mut parents = parents.to_vec();
        let self_name = parents.pop().unwrap_or("*anon*".to_string());
        parents.reverse();
        let path = if parents.len() > 0 {
            format!(" (included from {})",parents.join(", "))
        } else {
            String::new()
        };
        format!("{} at {}:{}{}",msg,self_name,line_no,path)
    } else {
        msg.to_string()
    }
}

pub trait PTTransformer {
    fn include(&mut self, _pos: (&[String],usize), _path: &str) -> Result<Option<Vec<PTStatement>>,String> { Ok(None) }
    fn remove_flags(&mut self, _flag: &str) -> Result<bool,String> { Ok(false) }
    fn bad_repeater(&mut self, _pos: (&[String],usize)) -> Result<(),String> { Ok(()) }
    fn call_to_expr(&mut self, _call: &PTCall, context: usize) -> Result<Option<PTExpression>,String> { Ok(None) }
    fn call_to_block(&mut self, _call: &PTCall, _pos: (&[String],usize), context: usize) -> Result<Option<Vec<PTStatement>>,String> { Ok(None) }
    fn replace_infix(&mut self, _a: &PTExpression, _f: &str, _b: &PTExpression) -> Result<Option<PTExpression>,String> { Ok(None) }
    fn replace_prefix(&mut self, _f: &str, _a: &PTExpression) -> Result<Option<PTExpression>,String> { Ok(None) }
}

#[derive(Debug,Clone)]
pub enum PTAtomicTypeSpec {
    Number,
    String,
    Boolean,
    Handle(String)
}

#[derive(Debug,Clone)]
pub enum PTTypeSpec {
    Atomic(PTAtomicTypeSpec),
    Sequence(PTAtomicTypeSpec),
    Wildcard(String),
    SequenceWildcard(String)
}

#[derive(Debug,Clone)]
pub enum PTConstant {
    Number(f64),
    String(String),
    Boolean(bool)
}

#[derive(Debug,Clone)]
pub struct PTCodeRegisterArgument {
    pub reg_id: usize,
    pub arg_types: Vec<PTTypeSpec>,
    pub checks: Vec<Check>
}

#[derive(Debug,Clone)]
pub struct PTFuncProcNamedArgument {
    pub id: String,
    pub arg_types: Vec<PTTypeSpec>,
    pub checks: Vec<Check>
}

#[derive(Debug,Clone)]
pub struct PTFuncProcAnonArgument {
    pub arg_types: Vec<PTTypeSpec>,
    pub checks: Vec<Check>
}

#[derive(Debug,Clone)]
pub enum PTFuncProcArgument {
    Named(PTFuncProcNamedArgument),
    Bundle(String)
}

#[derive(Debug,Clone)]
pub enum PTCodeArgument {
    Register(PTCodeRegisterArgument),
    Constant(PTConstant)
}

#[derive(Debug,Clone)]
pub enum PTCodeCommand {
    Opcode(usize,Vec<usize>),
    Register(usize)
}

#[derive(Debug,Clone)]
pub struct PTCodeBlock {
    pub name: String,
    pub arguments: Vec<PTCodeArgument>,
    pub results: Vec<PTCodeRegisterArgument>,
    pub commands: Vec<PTCodeCommand>,
    pub modifiers: Vec<CodeModifier>
}

impl PTCodeBlock {
    fn build(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Code(BTCodeDefinition::new(self.modifiers.clone())),bc)?;
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum PTCallArg {
    Expression(PTExpression),
    Bundle(String),
    Repeater(String)
}

impl PTCallArg {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: bool) -> Result<PTCallArg,String> {
        Ok(match self {
            PTCallArg::Expression(x) => PTCallArg::Expression(x.transform(transformer,pos,context,false)?),
            PTCallArg::Bundle(s) => PTCallArg::Bundle(s),
            PTCallArg::Repeater(r) => {
                if !top { transformer.bad_repeater(pos)?; }
                PTCallArg::Repeater(r)
            }
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTCall {
    pub name: String,
    pub args: Vec<PTCallArg>,
    pub is_macro: bool // removed in preprocessing
}

impl PTCall {
    fn transform(mut self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: bool) -> Result<PTCall,String> {
        let top = top && !self.is_macro;
        Ok(PTCall {
            name: self.name,
            args: self.args.drain(..).map(|x| x.transform(transformer,pos,context,top)).collect::<Result<Vec<_>,_>>()?,
            is_macro: self.is_macro
        })
    }

    fn transform_expression(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: bool) -> Result<PTExpression,String> {
        if let Some(repl) = transformer.call_to_expr(&self,context)? {
            Ok(repl)
        } else {
            Ok(PTExpression::Call(self.transform(transformer,pos,context,top)?))
        }
    }

    fn transform_block(&self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: bool) -> Result<Option<Vec<PTStatement>>,String> {
        if let Some(repl) = transformer.call_to_block(&self,pos,context)? {
            Ok(Some(repl))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug,Clone)]
pub enum PTLetAssign {
    Variable(Variable,Vec<Check>),
    Repeater(String)
}

impl PTLetAssign {
    fn is_repeater(&self) -> bool {
        match self {
            PTLetAssign::Repeater(_) => true,
            _ => false
        }
    }

    fn declare(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        match self {
            PTLetAssign::Variable(var,_) => {
                bt.declare(BTDeclare::Variable(var.clone()),bc)?;
            }
            PTLetAssign::Repeater(r) => {
                bt.declare(BTDeclare::Repeater(r.to_string()),bc)?;
            }
        }
        Ok(())
    }

    fn checks(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        match self {
            PTLetAssign::Variable(var,checks) => {
                for check in checks.iter() {
                    bt.check(var,check,bc)?;
                }
            },
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum PTProcAssignArg {
    Variable(Variable,Vec<Check>),
    Bundle(String)
}

impl PTProcAssignArg {
    fn declare(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        let declare = match self {
            PTProcAssignArg::Variable(v, _) => BTDeclare::Variable(v.clone()),
            PTProcAssignArg::Bundle(b) => BTDeclare::Bundle(b.to_string())
        };
        bt.declare(declare,bc)
    }

    fn checks(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        match self {
            PTProcAssignArg::Variable(var,checks) => {
                for check in checks.iter() {
                    bt.check(var,check,bc)?;
                }
            },
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum PTExpression {
    Constant(PTConstant),
    FiniteSequence(Vec<PTExpression>),
    InfiniteSequence(Box<PTExpression>),
    Variable(Variable),
    Infix(Box<PTExpression>,String,Box<PTExpression>),
    Prefix(String,Box<PTExpression>),
    Call(PTCall)
}

impl PTExpression {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: bool) -> Result<PTExpression,String> {
        Ok(match self {
            PTExpression::FiniteSequence(mut exprs) => {
                PTExpression::FiniteSequence(exprs.drain(..).map(|x| x.transform(transformer,pos,context,false)).collect::<Result<Vec<_>,_>>()?)
            },
            PTExpression::InfiniteSequence(expr) => {
                PTExpression::InfiniteSequence(Box::new(expr.transform(transformer,pos,context,false)?))
            },
            PTExpression::Infix(a,f,b) => {
                let a = a.transform(transformer,pos,context,false)?;
                let b = b.transform(transformer,pos,context,false)?;
                if let Some(repl) = transformer.replace_infix(&a,&f,&b)? {
                    repl
                } else {
                    PTExpression::Infix(Box::new(a),f,Box::new(b))
                }
            },
            PTExpression::Prefix(f,a) => {
                let a = a.transform(transformer,pos,context,false)?;
                if let Some(repl) = transformer.replace_prefix(&f,&a)? {
                    repl
                } else {
                    PTExpression::Prefix(f,Box::new(a))
                }
            },
            PTExpression::Call(call) => {
                call.transform_expression(transformer,pos,context,top)?
            },
            x => x
        })
    }
}

#[derive(Debug,Clone)]
pub enum PTFuncValue {
    Expression(PTExpression),
    Bundle(String)
}

impl PTFuncValue {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize) -> Result<PTFuncValue,String> {
        Ok(match self {
            PTFuncValue::Expression(expr) => PTFuncValue::Expression(expr.transform(transformer,pos,context,false)?),
            x => x
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTFuncDef {
    pub name: String,
    pub modifiers: Vec<FuncProcModifier>,
    pub args: Vec<PTFuncProcArgument>,
    pub block: Vec<PTStatement>,
    pub value: PTFuncValue,
    pub value_type: Option<PTFuncProcAnonArgument>
}

impl PTFuncDef {
    fn transform(mut self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize) -> Result<PTFuncDef,String> {
        Ok(PTFuncDef {
            name: self.name,
            args: self.args,
            modifiers: self.modifiers,
            block: PTStatement::transform_list(self.block,transformer)?,
            value: self.value.transform(transformer,pos,context)?,
            value_type: self.value_type
        })
    }

    fn build(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Func(),bc)?;
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum PTProcReturn {
    Named(Vec<PTExpression>),
    Bundle(String),
    None
}

impl PTProcReturn {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize) -> Result<PTProcReturn,String> {
        Ok(match self {
            PTProcReturn::Named(mut exprs) => {
                PTProcReturn::Named(exprs.drain(..).map(|x| x.transform(transformer,pos,context,false)).collect::<Result<Vec<_>,_>>()?)
            },
            x => x
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTProcDef {
    pub name: String,
    pub modifiers: Vec<FuncProcModifier>,
    pub args: Vec<PTFuncProcArgument>,
    pub block: Vec<PTStatement>,
    pub ret: PTProcReturn,
    pub ret_type: Option<Vec<PTFuncProcAnonArgument>>
}

impl PTProcDef {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize) -> Result<PTProcDef,String> {
        Ok(PTProcDef {
            name: self.name,
            args: self.args,
            modifiers: self.modifiers,
            block: PTStatement::transform_list(self.block,transformer)?,
            ret: self.ret.transform(transformer,pos,context)?,
            ret_type: self.ret_type
        })
    }

    fn build(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Proc(),bc)?;
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct PTStatement {
    pub value: PTStatementValue,
    pub file: Arc<Vec<String>>,
    pub line_no: usize,
    pub context: usize
}

#[derive(Debug,Clone)]
pub enum PTStatementValue {
    /* preprocessor */
    Include(String),
    Flag(String),

    /* definitions */
    Code(PTCodeBlock),
    FuncDef(PTFuncDef),
    ProcDef(PTProcDef),

    /* instructions */
    LetStatement(PTLetAssign,PTExpression),
    ModifyStatement(Variable,PTExpression),
    LetProcCall(Vec<PTProcAssignArg>,PTCall),
    ModifyProcCall(Vec<Variable>,PTCall),
    BareCall(PTCall),
}

impl PTStatement {
    pub fn transform(self, transformer: &mut dyn PTTransformer) -> Result<Vec<PTStatement>,String> {
        let pos = (self.file.as_ref().as_slice(),self.line_no);
        let value = match self.value {
            PTStatementValue::LetStatement(lvalue,rvalue) => {
                let top = lvalue.is_repeater();
                PTStatementValue::LetStatement(lvalue,rvalue.transform(transformer,pos,self.context,top)?)
            },
            PTStatementValue::ModifyStatement(lvalue,rvalue) => {
                PTStatementValue::ModifyStatement(lvalue,rvalue.transform(transformer,pos,self.context,false)?)
            },
            PTStatementValue::BareCall(call) => {
                if let Some(repl) = call.transform_block(transformer,pos,self.context,false)? {
                    return Ok(repl);
                } else {
                    PTStatementValue::BareCall(call)
                }
            },
            PTStatementValue::FuncDef(call) => {
                PTStatementValue::FuncDef(call.transform(transformer,pos,self.context)?)
            },
            PTStatementValue::ProcDef(call) => {
                PTStatementValue::ProcDef(call.transform(transformer,pos,self.context)?)
            },
            x => x
        };
        Ok(vec![PTStatement {
            value,
            file: self.file,
            line_no: self.line_no,
            context: self.context
        }])
    }

    pub fn transform_list(this: Vec<Self>, transformer: &mut dyn PTTransformer) -> Result<Vec<PTStatement>,String> {
        let mut out = vec![];
        for block in this {
            let pos = (block.file.as_ref().as_slice(),block.line_no);
            let mut more = match &block.value {
                PTStatementValue::Include(path) => {
                    if pos.0.contains(path) {
                        return Err(at(&format!("recursive include of {}",path),Some(pos)))
                    }
                    if let Some(repl) = transformer.include(pos,path)? {
                        repl
                    } else {
                        vec![block]
                    }
                },
                PTStatementValue::Flag(flag) => {
                    if transformer.remove_flags(&flag)? {
                        vec![]
                    } else {
                        vec![block]
                    }
                },    
                _ => block.transform(transformer)?
            };
            out.append(&mut more);
        }
        Ok(out)    
    }

    fn build(&self, bt: &mut BuildTree, bc: &BuildContext) -> Result<(),String> {
        match &self.value {
            PTStatementValue::Include(_) |
            PTStatementValue::Flag(_) => {
                panic!("item should have been eliminated from build tree");
            },

            PTStatementValue::FuncDef(f) => { f.build(bt,bc)?; },
            PTStatementValue::ProcDef(p) => { p.build(bt,bc)?; }
            PTStatementValue::Code(c) => { c.build(bt,bc)?; },

            PTStatementValue::LetStatement(v,x) => {
                v.declare(bt,bc)?;
                v.checks(bt,bc)?;
            },
            PTStatementValue::LetProcCall(args,x) => {
                for arg in args {
                    arg.declare(bt,bc)?;
                }
                for arg in args {
                   arg.checks(bt,bc)?;
                }
            },
            PTStatementValue::ModifyProcCall(_,_) => {},
            PTStatementValue::ModifyStatement(_, _) => {},
            PTStatementValue::BareCall(_) => {},
        }
        Ok(())
    }

    pub(crate) fn to_build_tree(this: Vec<Self>) -> Result<BuildTree,String> {
        let mut bt = BuildTree::new();
        let mut bc = BuildContext::new();
        for stmt in this.iter() {
            bc.set_location(&stmt.file,stmt.line_no);
            stmt.build(&mut bt,&mut bc).map_err(|e| {
                at(&e,Some(bc.location()))
            })?;
        }
        Ok(bt)
    }
}
