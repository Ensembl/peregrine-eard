use std::sync::Arc;

use crate::{buildtree::{BuildTree, BTDefinition, BTCodeDefinition, BTDeclare, BuildContext, BTExpression, BTFuncCall}, model::{CodeModifier, Variable, Check, FuncProcModifier, CallArg, Constant}};

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
    Constant(Constant)
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
    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Code(BTCodeDefinition::new(self.modifiers.clone())),bc,false)?;
        Ok(())
    }
}

impl CallArg<PTExpression> {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, allow_repeat: Option<&mut bool>) -> Result<CallArg<PTExpression>,String> {
        Ok(match self {
            CallArg::Expression(x) => CallArg::Expression(x.transform(transformer,pos,context,None)?),
            CallArg::Bundle(s) => CallArg::Bundle(s),
            CallArg::Repeater(r) => {
                let mut bad = true;
                if let Some(allow_repeat) = allow_repeat {
                    if *allow_repeat {
                        *allow_repeat = false;
                        bad = false;
                    }
                }
                if bad { transformer.bad_repeater(pos)?; }
                CallArg::Repeater(r)
            }
        })
    }

    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<CallArg<BTExpression>,String> {
        Ok(match self {
            CallArg::Expression(x) => CallArg::Expression(x.build(bt,bc)?),
            CallArg::Bundle(n) => CallArg::Bundle(n.to_string()),
            CallArg::Repeater(n) => CallArg::Repeater(n.to_string())
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTCall {
    pub name: String,
    pub args: Vec<CallArg<PTExpression>>,
    pub is_macro: bool // removed in preprocessing
}

impl PTCall {
    fn transform(mut self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, allow_repeat: Option<&mut bool>) -> Result<PTCall,String> {
        let mut no_repeat = false;
        let allow_repeat = allow_repeat.unwrap_or(&mut no_repeat);
        let mut args = vec![];
        for arg in self.args {
            args.push(arg.transform(transformer,pos,context,Some(allow_repeat))?);
        }
        Ok(PTCall {
            name: self.name,
            args,
            is_macro: self.is_macro
        })
    }

    fn transform_expression(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, top: Option<&mut bool>) -> Result<PTExpression,String> {
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

    fn build_proc(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<(),String> {
        let index = bc.lookup(&self.name)?;
        let args = self.args.iter().map(|a| a.build(bt,bc)).collect::<Result<_,_>>()?;
        bt.statement(index,args,bc)?;
        Ok(())
    }

    fn build_func(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<BTFuncCall,String> {
        let index = bc.lookup(&self.name)?;
        let args = self.args.iter().map(|a| a.build(bt,bc)).collect::<Result<_,_>>()?;
        Ok(bt.function_call(index,args,bc)?)
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
    Constant(Constant),
    FiniteSequence(Vec<PTExpression>),
    InfiniteSequence(Box<PTExpression>),
    Variable(Variable),
    Infix(Box<PTExpression>,String,Box<PTExpression>), // replaced during prerpocessing
    Prefix(String,Box<PTExpression>), // replaced during prerpocessing
    Call(PTCall)
}

impl PTExpression {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: (&[String],usize), context: usize, allow_repeat: Option<&mut bool>) -> Result<PTExpression,String> {
        let mut no_repeats = false;
        Ok(match self {
            PTExpression::FiniteSequence(mut exprs) => {
                PTExpression::FiniteSequence(exprs.drain(..).map(|x| x.transform(transformer,pos,context,None)).collect::<Result<Vec<_>,_>>()?)
            },
            PTExpression::InfiniteSequence(expr) => {
                PTExpression::InfiniteSequence(Box::new(expr.transform(transformer,pos,context,None)?))
            },
            PTExpression::Infix(a,f,b) => {
                let a = a.transform(transformer,pos,context,None)?;
                let b = b.transform(transformer,pos,context,None)?;
                if let Some(repl) = transformer.replace_infix(&a,&f,&b)? {
                    repl
                } else {
                    PTExpression::Infix(Box::new(a),f,Box::new(b))
                }
            },
            PTExpression::Prefix(f,a) => {
                let a = a.transform(transformer,pos,context,None)?;
                if let Some(repl) = transformer.replace_prefix(&f,&a)? {
                    repl
                } else {
                    PTExpression::Prefix(f,Box::new(a))
                }
            },
            PTExpression::Call(call) => {
                call.transform_expression(transformer,pos,context,allow_repeat)?
            },
            x => x
        })
    }

    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<BTExpression,String> {
        Ok(match self {
            PTExpression::Constant(c) => BTExpression::Constant(c.clone()),

            PTExpression::FiniteSequence(items) => {
                let reg = bc.allocate_register();
                bt.statement(bc.lookup("__operator_finseq")?, vec![], bc)?; // XXX out to reg
                for item in items {
                    let built = item.build(bt,bc)?;
                    bt.statement(bc.lookup("__operator_push")?, vec![
                        CallArg::Expression(BTExpression::RegisterValue(reg)),
                        CallArg::Expression(built)
                    ], bc)?;
                }
                BTExpression::RegisterValue(reg)
            },
            PTExpression::InfiniteSequence(x) => {
                BTExpression::Function(BTFuncCall {
                    func_index: bc.lookup("__operator_infseq")?,
                    args: vec![CallArg::Expression(x.build(bt,bc)?)],
                })
            },

            PTExpression::Variable(v) => BTExpression::Variable(v.clone()),
            PTExpression::Call(c) => {
                BTExpression::Function(c.build_func(bt,bc)?)
            },

            PTExpression::Infix(_, n, _) |
            PTExpression::Prefix(n, _) => {
                panic!("operator {} should have been replaced during preprocessing!",n);
            }
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
            PTFuncValue::Expression(expr) => PTFuncValue::Expression(expr.transform(transformer,pos,context,None)?),
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

    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Func(),bc,self.modifiers.contains(&FuncProcModifier::Export))?;
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
                PTProcReturn::Named(exprs.drain(..).map(|x| x.transform(transformer,pos,context,None)).collect::<Result<Vec<_>,_>>()?)
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

    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<(),String> {
        bt.define(&self.name,BTDefinition::Proc(),bc,self.modifiers.contains(&FuncProcModifier::Export))?;
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
    LetStatement(Vec<PTLetAssign>,Vec<PTExpression>),
    ModifyStatement(Variable,PTExpression),
    LetProcCall(Vec<PTProcAssignArg>,PTCall),
    ModifyProcCall(Vec<Variable>,PTCall),
    BareCall(PTCall),
}

impl PTStatement {
    pub fn transform(self, transformer: &mut dyn PTTransformer) -> Result<Vec<PTStatement>,String> {
        let pos = (self.file.as_ref().as_slice(),self.line_no);
        let value = match self.value {
            PTStatementValue::LetStatement(lvalues,rvalues) => {
                let count_repeats = lvalues.iter().map(|x| x.is_repeater()).count();
                if count_repeats > 1 {
                    return Err("only one repeat permitted per statement".to_string());
                }
                let mut allow_repeat = count_repeats > 0;
                let mut exprs = vec![];
                for rvalue in rvalues {
                    exprs.push(rvalue.transform(transformer,pos,self.context,Some(&mut allow_repeat))?);
                }
                PTStatementValue::LetStatement(lvalues,exprs)
            },
            PTStatementValue::ModifyStatement(lvalue,rvalue) => {
                PTStatementValue::ModifyStatement(lvalue,rvalue.transform(transformer,pos,self.context,None)?)
            },
            PTStatementValue::BareCall(call) => {
                if let Some(repl) = call.transform_block(transformer,pos,self.context,false)? {
                    return Ok(repl);
                } else {
                    PTStatementValue::BareCall(call.transform(transformer,pos,self.context,None)?)
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

    fn build(&self, bt: &mut BuildTree, bc: &mut BuildContext) -> Result<(),String> {
        match &self.value {
            PTStatementValue::Include(_) |
            PTStatementValue::Flag(_) => {
                panic!("item should have been eliminated from build tree");
            },

            PTStatementValue::FuncDef(f) => { f.build(bt,bc)?; },
            PTStatementValue::ProcDef(p) => { p.build(bt,bc)?; }
            PTStatementValue::Code(c) => { c.build(bt,bc)?; },

            PTStatementValue::LetStatement(vv,xx) => {
                if vv.len() != xx.len() {
                    return Err(format!("let tuples differ in length"));
                }
                for (v,x) in vv.iter().zip(xx.iter()) {
                    v.declare(bt,bc)?;
                }
                for (v,x) in vv.iter().zip(xx.iter()) {
                    v.checks(bt,bc)?;
                }
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
            PTStatementValue::BareCall(c) => {
                c.build_proc(bt,bc)?;
            },
        }
        Ok(())
    }

    pub(crate) fn to_build_tree(this: Vec<Self>) -> Result<BuildTree,String> {
        let mut bt = BuildTree::new();
        let mut bc = BuildContext::new();
        for stmt in this.iter() {
            bc.set_location(&stmt.file,stmt.line_no);
            bc.set_file_context(stmt.context);
            stmt.build(&mut bt,&mut bc).map_err(|e| {
                at(&e,Some(bc.location()))
            })?;
        }
        Ok(bt)
    }
}
