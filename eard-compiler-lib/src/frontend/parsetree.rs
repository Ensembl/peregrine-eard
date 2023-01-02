use crate::{model::{constants::Constant, checkstypes::{TypedArgument, ArgTypeSpec, Check}, codeblocks::CodeBlock}, controller::source::ParsePosition};
use super::{buildtree::{BuildTree, Variable}, femodel::{OrBundle, OrBundleRepeater}};
use super::buildtreebuilder::BuildContext;

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

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum FuncProcModifier {
    Export,
    Entry,
    Version(Vec<String>)
}

pub trait PTTransformer {
    fn include(&mut self, _pos: &ParsePosition, _fixed: bool, _path: &str) -> Result<Option<Vec<PTStatement>>,String> { Ok(None) }
    fn bad_repeater(&mut self, _pos: (&[String],usize)) -> Result<(),String> { Ok(()) }
    fn call_to_expr(&mut self, _call: &PTCall, _context: usize) -> Result<Option<PTExpression>,String> { Ok(None) }
    fn call_to_block(&mut self, _call: &PTCall, _pos: &ParsePosition, _context: usize) -> Result<Option<Vec<PTStatement>>,String> { Ok(None) }
    fn replace_infix(&mut self, _a: &PTExpression, _f: &str, _b: &PTExpression) -> Result<Option<PTExpression>,String> { Ok(None) }
    fn replace_prefix(&mut self, _f: &str, _a: &PTExpression) -> Result<Option<PTExpression>,String> { Ok(None) }
}

impl OrBundleRepeater<PTExpression> {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<OrBundleRepeater<PTExpression>,String> {
        Ok(match self {
            OrBundleRepeater::Normal(x) => OrBundleRepeater::Normal(x.transform(transformer,pos,context)?),
            OrBundleRepeater::Bundle(s) => OrBundleRepeater::Bundle(s),
            OrBundleRepeater::Repeater(r) => OrBundleRepeater::Repeater(r)
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTCall {
    pub name: String,
    pub args: Vec<OrBundleRepeater<PTExpression>>,
    pub is_macro: bool // removed in preprocessing
}

impl PTCall {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<PTCall,String> {
        let mut args = vec![];
        for arg in self.args {
            args.push(arg.transform(transformer,pos,context)?);
        }
        Ok(PTCall {
            name: self.name,
            args,
            is_macro: self.is_macro
        })
    }

    fn transform_expression(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<PTExpression,String> {
        if let Some(repl) = transformer.call_to_expr(&self,context)? {
            Ok(repl)
        } else {
            Ok(PTExpression::Call(self.transform(transformer,pos,context)?))
        }
    }

    fn transform_block(&self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<Option<Vec<PTStatement>>,String> {
        if let Some(repl) = transformer.call_to_block(&self,pos,context)? {
            Ok(Some(repl))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug,Clone)]
pub enum PTExpression {
    Constant(Constant),
    FiniteSequence(Vec<PTExpression>),
    InfiniteSequence(Box<PTExpression>),
    Variable(Variable),
    Infix(Box<PTExpression>,String,Box<PTExpression>), // replaced during preprocessing
    Prefix(String,Box<PTExpression>), // replaced during preprocessing
    Call(PTCall)
}

impl PTExpression {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<PTExpression,String> {
        Ok(match self {
            PTExpression::FiniteSequence(mut exprs) => {
                PTExpression::FiniteSequence(exprs.drain(..).map(|x| x.transform(transformer,pos,context)).collect::<Result<Vec<_>,_>>()?)
            },
            PTExpression::InfiniteSequence(expr) => {
                PTExpression::InfiniteSequence(Box::new(expr.transform(transformer,pos,context)?))
            },
            PTExpression::Infix(a,f,b) => {
                let a = a.transform(transformer,pos,context)?;
                let b = b.transform(transformer,pos,context)?;
                if let Some(repl) = transformer.replace_infix(&a,&f,&b)? {
                    repl
                } else {
                    PTExpression::Infix(Box::new(a),f,Box::new(b))
                }
            },
            PTExpression::Prefix(f,a) => {
                let a = a.transform(transformer,pos,context)?;
                if let Some(repl) = transformer.replace_prefix(&f,&a)? {
                    repl
                } else {
                    PTExpression::Prefix(f,Box::new(a))
                }
            },
            PTExpression::Call(call) => {
                call.transform_expression(transformer,pos,context)?
            },
            x => x
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTFuncDef {
    pub name: String,
    pub modifiers: Vec<FuncProcModifier>,
    pub args: Vec<OrBundle<TypedArgument>>,
    pub captures: Vec<OrBundle<Variable>>,
    pub block: Vec<PTStatement>,
    pub value: OrBundle<PTExpression>,
    pub value_type: Option<ArgTypeSpec>
}

impl PTFuncDef {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<PTFuncDef,String> {
        Ok(PTFuncDef {
            name: self.name,
            args: self.args,
            modifiers: self.modifiers,
            captures: self.captures,
            block: PTStatement::transform_list(self.block,transformer)?,
            value: self.value.transform(transformer,pos,context)?,
            value_type: self.value_type
        })
    }

    pub(crate) fn versions(&self) -> Vec<Vec<String>> {
        self.modifiers.iter().filter_map(|m| {
            match m {
                FuncProcModifier::Version(v) => Some(v.to_vec()),
                _ => None
            }
        }).collect()
    }
}

impl OrBundle<PTExpression> {
    fn transform(self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<OrBundle<PTExpression>,String> {
        Ok(match self {
            OrBundle::Normal(expr) => {
                OrBundle::Normal(expr.transform(transformer,pos,context)?)
            },
            x => x
        })
    }
}

#[derive(Debug,Clone)]
pub struct PTProcDef {
    pub name: String,
    pub modifiers: Vec<FuncProcModifier>,
    pub args: Vec<OrBundle<TypedArgument>>,
    pub captures: Vec<OrBundle<Variable>>,
    pub block: Vec<PTStatement>,
    pub ret: Vec<OrBundle<PTExpression>>,
    pub ret_type: Option<Vec<ArgTypeSpec>>
}

impl PTProcDef {
    fn transform(mut self, transformer: &mut dyn PTTransformer, pos: &ParsePosition, context: usize) -> Result<PTProcDef,String> {
        Ok(PTProcDef {
            name: self.name,
            args: self.args,
            modifiers: self.modifiers,
            captures: self.captures,
            block: PTStatement::transform_list(self.block,transformer)?,
            ret: self.ret.drain(..).map(|x| x.transform(transformer,pos,context)).collect::<Result<_,_>>()?,
            ret_type: self.ret_type
        })
    }

    pub(crate) fn versions(&self) -> Vec<Vec<String>> {
        self.modifiers.iter().filter_map(|m| {
            match m {
                FuncProcModifier::Version(v) => Some(v.to_vec()),
                _ => None
            }
        }).collect()
    }
}

#[derive(Clone,PartialEq,Eq)]
pub enum CodeArgModifier {
    Large,
    Sparse
}

#[derive(Debug,Clone)]
pub struct PTStatement {
    pub value: PTStatementValue,
    pub position: ParsePosition,
    pub context: usize
}

#[derive(Debug,Clone)]
pub enum PTStatementValue {
    /* preprocessor */
    Include(String,bool),
    Header(String,String,u32),
    Version(String,u32,u32),

    /* definitions */
    Code(CodeBlock),
    FuncDef(PTFuncDef),
    ProcDef(PTProcDef),

    /* instructions */
    LetStatement(Vec<OrBundleRepeater<(Variable,Vec<Check>)>>,Vec<OrBundle<PTExpression>>),
    ModifyStatement(Vec<Variable>,Vec<PTExpression>),
    Expression(PTExpression),
    MacroCall(PTCall),
}

impl PTStatement {
    pub fn transform(self, transformer: &mut dyn PTTransformer) -> Result<Vec<PTStatement>,String> {
        let pos = self.position.clone();
        let value = match self.value {
            PTStatementValue::LetStatement(lvalues,rvalues) => {
                let count_repeats = lvalues.iter().filter(|x| x.is_repeater()).count();
                if count_repeats > 1 {
                    return Err(format!("only one repeat permitted per statement"));
                }
                let mut exprs = vec![];
                for rvalue in rvalues {
                    exprs.push(rvalue.transform(transformer,&pos,self.context)?);
                }
                PTStatementValue::LetStatement(lvalues,exprs)
            },
            PTStatementValue::ModifyStatement(lvalues,mut rvalues) => {
                let context = self.context.clone();
                let rvalues = rvalues.drain(..).map(|x| 
                    x.transform(transformer,&pos,context)
                ).collect::<Result<_,_>>()?;
                PTStatementValue::ModifyStatement(lvalues,rvalues)
            },
            PTStatementValue::MacroCall(call) => {
                if let Some(repl) = call.transform_block(transformer,&pos,self.context)? {
                    return Ok(repl);
                } else {
                    return Err(format!("no such macro"));
                }
            },
            PTStatementValue::Expression(x) => {
                let context = self.context.clone();
                PTStatementValue::Expression(x.transform(transformer,&pos,context)?)
            },
            PTStatementValue::FuncDef(call) => {
                PTStatementValue::FuncDef(call.transform(transformer,&pos,self.context)?)
            },
            PTStatementValue::ProcDef(call) => {
                PTStatementValue::ProcDef(call.transform(transformer,&pos,self.context)?)
            },
            x => x
        };
        Ok(vec![PTStatement {
            value,
            position: pos.clone(),
            context: self.context
        }])
    }

    pub fn transform_list(this: Vec<Self>, transformer: &mut dyn PTTransformer) -> Result<Vec<PTStatement>,String> {
        let mut out = vec![];
        for block in this {
            let pos = block.position.clone();
            let mut more = match &block.value {
                PTStatementValue::Include(path,fixed) => {
                    if pos.contains(path) {
                        return Err(pos.message(&format!("recursive include of {}",path)));
                    }
                    if let Some(repl) = transformer.include(&pos,*fixed,path)? {
                        repl
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

    pub(crate) fn to_build_tree(this: Vec<Self>, target_version: Option<u32>) -> Result<BuildTree,String> {
        let mut bt = BuildTree::new();
        let mut bc = BuildContext::new(target_version);
        for stmt in this.iter() {
            bc.set_location(&stmt.position);
            bc.set_file_context(stmt.context);
            bc.build_statement(&mut bt,&stmt).map_err(|e| {
                bc.location().message(&e)
            })?;
        }
        bt.finish();
        Ok(bt)
    }
}
