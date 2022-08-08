use std::{sync::Arc, collections::BTreeMap};

use crate::{model::{OrBundle, TypedArgument, ArgTypeSpec, CodeBlock, CallArg, FuncProcModifier, Variable}, buildtree::{BuildTree, BTStatementValue, BTStatement, BTExpression, BTDefinitionVariety, BTFuncProcDefinition, BTDefinition, BTCodeDefinition, BTLValue, BTFuncCall, BTDeclare}, parsetree::{PTExpression, PTCall, PTStatement, PTStatementValue, PTLetAssign, PTFuncDef, PTProcDef}};

#[derive(Debug,Clone)]
pub(super) enum DefName {
    Func(usize),
    Proc(usize),
    Code(usize)
}

struct CurrentFuncProcDefinition {
    block: Vec<BTStatement>,
    name: String,
    export: bool,
    args: Vec<OrBundle<TypedArgument>>,
    captures: Vec<OrBundle<Variable>>,
    variety: BTDefinitionVariety,
    ret_type: Option<Vec<ArgTypeSpec>>, // exactly one for functions
}

impl CurrentFuncProcDefinition {
    fn to_funcproc(&self, ret: &[OrBundle<BTExpression>]) -> BTFuncProcDefinition {
        BTFuncProcDefinition {
            args: self.args.clone(),
            captures: self.captures.clone(),
            ret_type: self.ret_type.clone(),
            ret: ret.to_vec(),
            block: self.block.clone()
        }
    }
}

pub struct BuildContext {
    location: (Arc<Vec<String>>,usize),
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),DefName>,
    next_register: usize,
    funcproc_target: Option<CurrentFuncProcDefinition>
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext {
            location: (Arc::new(vec!["*anon*".to_string()]),0),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0,
            funcproc_target: None
        }
    }

    pub fn push_funcproc_target(&mut self, is_proc: bool, name: &str,
            args: &[OrBundle<TypedArgument>],
            ret_type: Option<Vec<ArgTypeSpec>>,
            captures: &[OrBundle<Variable>],
            export: bool, bt: &mut BuildTree) {
        let variety = if is_proc { BTDefinitionVariety::Proc } else { BTDefinitionVariety::Func };
        self.funcproc_target = Some(CurrentFuncProcDefinition {
            name: name.to_string(),
            args: args.to_vec(),
            captures: captures.to_vec(),
            export, variety,
            block: vec![],
            ret_type
        });
    }

    pub fn pop_funcproc_target(&mut self, ret: &[OrBundle<BTExpression>], bt: &mut BuildTree) -> Result<(),String> {
        let ctx = self.funcproc_target.take().expect("pop without push");
        let defn = match &ctx.variety {
            BTDefinitionVariety::Func => {                
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret),false,bt,ctx.export)?
            },
            BTDefinitionVariety::Proc => {
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret),true,bt,ctx.export)?
            }
        };
        self.add_statement(bt,defn);
        Ok(())
    }

    pub fn set_file_context(&mut self, context: usize) {
        self.file_context = context;
    }

    pub fn set_location(&mut self, file: &Arc<Vec<String>>, line_no: usize) {
        self.location = (file.clone(),line_no);
    }

    pub fn location(&self) -> (&[String],usize) {
        (self.location.0.as_ref(),self.location.1)
    }

    fn lookup(&self, name: &str) -> Result<DefName,String> {
        self.defnames
            .get(&(Some(self.file_context),name.to_string()))
            .or_else(||
                self.defnames.get(&(None,name.to_string()))
            )
            .ok_or_else(|| format!("No such function/procedure {}",name))
            .cloned()
    }

    pub(crate) fn lookup_func(&self, name: &str) -> Result<usize,String> {
        match self.lookup(name) {
            Ok(DefName::Func(x)) => Ok(x),
            _ => Err(format!("cannot find function {}",name))
        }
    }

    pub(crate) fn lookup_proc(&self, name: &str) -> Result<usize,String> {
        match self.lookup(name) {
            Ok(DefName::Proc(x)) => Ok(x),
            _ => Err(format!("cannot find procedure {}",name))
        }
    }

    pub(crate) fn allocate_register(&mut self) -> usize {
        self.next_register += 1;
        self.next_register
    }

    pub(super) fn define_funcproc(&mut self, name: &str, definition: BTFuncProcDefinition, is_proc: bool, bt: &mut BuildTree, export: bool) -> Result<BTStatementValue,String> {
        let definition = if is_proc { BTDefinition::Proc(definition) } else { BTDefinition::Func(definition) };
        let id = bt.add_definition(definition);
        let name_id = if is_proc { DefName::Proc(id) } else { DefName::Func(id) };
        let context = if export { None } else { Some(self.file_context) };
        let key = (context,name.to_string());
        if self.defnames.contains_key(&key) {
            return Err(format!("duplciate definition for {}",name));
        }
        self.defnames.insert(key,name_id);
        Ok(BTStatementValue::Define(id))
    }

    pub(crate) fn define_code(&mut self, block: &CodeBlock, bt: &mut BuildTree) -> Result<(),String> {
        let key = (Some(self.file_context),block.name.to_string());
        if !self.defnames.contains_key(&key) {
            let id = bt.add_definition(BTDefinition::Code(BTCodeDefinition {
                blocks: vec![]
            }));
            let name_id = DefName::Code(id);
            self.defnames.insert(key.clone(),name_id);
        }
        match self.defnames.get(&key) {
            Some(DefName::Code(id)) => {
                bt.add_code(*id,block)?;
                Ok(())
            },
            _ => {
                Err(format!("duplciate definition for {}",block.name))
            }
        }
    }

    pub(crate) fn add_statement(&mut self, bt: &mut BuildTree, value: BTStatementValue) -> Result<(),String> {
        let stmt = BTStatement {
            value,
            file: self.location.0.clone(),
            line_no: self.location.1
        };
        if let Some(target) = self.funcproc_target.as_mut() {
            target.block.push(stmt);
        } else {
            bt.add_statement(stmt);
        }
        Ok(())
    }

    fn is_procedure(&self, x: &OrBundle<PTExpression>, bt: &mut BuildTree) -> Result<bool,String> {
        Ok(match x {
            OrBundle::Normal(PTExpression::Call(c)) => {
                match self.lookup(&c.name)? {
                    DefName::Proc(_) => true,
                    _ => false
                }
            },
            _ => false
        })
    }

    fn lvalue_to_rvalue(&self, lvalue: &BTLValue) -> Result<CallArg<BTExpression>,String> {
        let expr = match lvalue {
            BTLValue::Variable(v) => BTExpression::Variable(v.clone()),
            BTLValue::Bundle(b) => BTExpression::Bundle(b.to_string()),
            BTLValue::Register(r) => BTExpression::RegisterValue(*r),
            BTLValue::Repeater(r) => {return Ok(CallArg::Repeater(r.to_string())); }
        };
        Ok(CallArg::Expression(expr))
    }

    fn make_statement(&mut self, vv: &[PTLetAssign], xx: &[OrBundle<PTExpression>], declare: bool, bt: &mut BuildTree) -> Result<(),String> {
        /* Step 1: allocate temporaries */
        let mut regs = vec![];
        for v in vv.iter() {
            match v {
                PTLetAssign::Variable(_,_) => { regs.push(BTLValue::Register(self.allocate_register())); },
                PTLetAssign::Bundle(b) => { regs.push(BTLValue::Bundle(b.to_string())); },
                PTLetAssign::Repeater(r) => { regs.push(BTLValue::Repeater(r.to_string())); },
            }
        }
        /* Step 2: evaluate expressions and put into temporaries */
        if xx.len() == 1 && self.is_procedure(&xx[0],bt)? {
            match &xx[0] {
                OrBundle::Normal(PTExpression::Call(call)) => {
                    self.build_proc(Some(regs.clone()),bt,call)?;
                },
                _ => {
                    return Err(format!("let tuples differ in length: {} lvalues but {} rvalues",vv.len(),xx.len()));
                }
            }
        } else if vv.len() == xx.len() {
            /* Lengths equal, so pair off */
            for (x,reg) in xx.iter().zip(regs.clone().iter()) {
                let call_arg = match x {
                    OrBundle::Normal(x) => {
                        let expr = self.build_expression(bt,x)?;    
                        CallArg::Expression(expr)
                    },
                    OrBundle::Bundle(b) => {
                        CallArg::Bundle(b.to_string())
                    }
                };
                let stmt = bt.statement(None,vec![call_arg],Some(vec![reg.clone()]))?;
                self.add_statement(bt,stmt)?;
            }
        } else {
            return Err(format!("let tuples differ in length: {} lvalues but {} rvalues",vv.len(),xx.len()));
        }
        if declare {
            /* Step 3: declare variables */
            for v in vv.iter() {
                self.build_declare(bt,v)?;
            }
        }
        /* Step 4: assign from temporaries to variables */
        for (v,reg) in vv.iter().zip(regs.iter()) {
            let lvalue = match v {
                PTLetAssign::Variable(v,_) => Some(BTLValue::Variable(v.clone())),
                PTLetAssign::Bundle(_) => None,
                PTLetAssign::Repeater(_) => None
            };
            if let Some(lvalue) = lvalue {
                let stmt = bt.statement(None,vec![
                    self.lvalue_to_rvalue(reg)?
                ],Some(vec![lvalue]))?;
                self.add_statement(bt,stmt)?;    
            }
        }
        /* Step 5: check sizes */
        for v in vv.iter() {
            self.build_checks(bt,v)?;
        }
        Ok(())
    }
    
    pub(crate) fn build_call(&mut self, bt: &mut BuildTree, expr: CallArg<PTExpression>) -> Result<CallArg<BTExpression>,String> {
        Ok(match expr {
            CallArg::Expression(x) => CallArg::Expression(self.build_expression(bt,&x)?),
            CallArg::Bundle(n) => CallArg::Bundle(n.to_string()),
            CallArg::Repeater(n) => CallArg::Repeater(n.to_string())
        })
    }

    fn build_proc(&mut self, rets: Option<Vec<BTLValue>>, bt: &mut BuildTree, call: &PTCall) -> Result<(),String> {
        let index = self.lookup_proc(&call.name)?;
        let args = call.args.iter().map(|a| self.build_call(bt,a.clone())).collect::<Result<_,_>>()?;
        let stmt = bt.statement(Some(index),args,rets)?;
        self.add_statement(bt,stmt)?;
        Ok(())
    }

    fn build_func(&mut self, bt: &mut BuildTree, call: &PTCall) -> Result<BTFuncCall,String> {
        let index = self.lookup_func(&call.name)?;
        let args = call.args.iter().map(|a| self.build_call(bt,a.clone())).collect::<Result<_,_>>()?;
        Ok(bt.function_call(index,args)?)
    }

    fn build_declare(&mut self, bt: &mut BuildTree, decl: &PTLetAssign) -> Result<(),String> {
        let stmt = BTStatementValue::Declare(match decl {
            PTLetAssign::Variable(var,_) => {
                BTDeclare::Variable(var.clone())
            },
            PTLetAssign::Bundle(b) => {
                BTDeclare::Bundle(b.to_string())
            },
            PTLetAssign::Repeater(r) => {
               BTDeclare::Repeater(r.to_string())
            }
        });
        self.add_statement(bt,stmt)?;
        Ok(())
    }

    fn build_checks(&mut self, bt: &mut BuildTree, decl: &PTLetAssign) -> Result<(),String> {
        match decl {
            PTLetAssign::Variable(var,checks) => {
                for check in checks.iter() {
                    let stmt = BTStatementValue::Check(var.clone(),check.clone());
                    self.add_statement(bt,stmt)?;
                }
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn build_expression(&mut self, bt: &mut BuildTree, expr: &PTExpression) -> Result<BTExpression,String> {
        Ok(match expr {
            PTExpression::Constant(c) => BTExpression::Constant(c.clone()),

            PTExpression::FiniteSequence(items) => {
                let reg = self.allocate_register();
                let stmt = bt.statement(Some(self.lookup_func("__operator_finseq")?), vec![], Some(vec![BTLValue::Register(reg)]))?; // XXX out to reg
                self.add_statement(bt,stmt)?;
                for item in items {
                    let built = self.build_expression(bt,item)?;
                    let stmt = bt.statement(Some(self.lookup_proc("__operator_push")?), vec![
                        CallArg::Expression(BTExpression::RegisterValue(reg)),
                        CallArg::Expression(built)
                    ], Some(vec![BTLValue::Register(reg)]))?;
                    self.add_statement(bt,stmt)?;
                }
                BTExpression::RegisterValue(reg)
            },
            PTExpression::InfiniteSequence(x) => {
                BTExpression::Function(BTFuncCall {
                    func_index: self.lookup_func("__operator_infseq")?,
                    args: vec![CallArg::Expression(self.build_expression(bt,x)?)],
                })
            },

            PTExpression::Variable(v) => BTExpression::Variable(v.clone()),
            PTExpression::Call(c) => {
                BTExpression::Function(self.build_func(bt,c)?)
            },

            PTExpression::Infix(_, n, _) |
            PTExpression::Prefix(n, _) => {
                panic!("operator {} should have been replaced during preprocessing!",n);
            }
        })
    }

    fn build_expr_ob(&mut self, bt: &mut BuildTree, expr: &OrBundle<PTExpression>) -> Result<OrBundle<BTExpression>,String> {
        Ok(match expr {
            OrBundle::Normal(n) => OrBundle::Normal(self.build_expression(bt,n)?),
            OrBundle::Bundle(b) => OrBundle::Bundle(b.clone())
        })
    }

    fn build_funcdef(&mut self, bt: &mut BuildTree, def: &PTFuncDef) -> Result<(),String> {
        let ret_type = def.value_type.as_ref().map(|x| vec![x.clone()]);
        let export = def.modifiers.contains(&FuncProcModifier::Export);
        self.push_funcproc_target(false,&def.name,&def.args,ret_type,&def.captures,export,bt);
        for stmt in &def.block {
            self.build_statement(bt,stmt)?;
        }
        let expr = self.build_expr_ob(bt,&def.value)?;
        self.pop_funcproc_target(&[expr],bt)?;
        Ok(())
    }

    fn build_procdef(&mut self, bt: &mut BuildTree, def: &PTProcDef) -> Result<(),String> {
        let export = def.modifiers.contains(&FuncProcModifier::Export);
        self.push_funcproc_target(true,&def.name,&def.args,def.ret_type.clone(),&def.captures,export,bt);
        for stmt in &def.block {
            self.build_statement(bt,stmt)?;
        }
        let ret = def.ret.iter().map(|x| self.build_expr_ob(bt,x)).collect::<Result<Vec<_>,_>>()?;
        self.pop_funcproc_target(&ret,bt)?;
        Ok(())
    }

    pub(super) fn build_statement(&mut self, bt: &mut BuildTree, stmt: &PTStatement) -> Result<(),String> {
        match &stmt.value {
            PTStatementValue::Include(_) |
            PTStatementValue::Flag(_) => {
                panic!("item should have been eliminated from build tree");
            },

            PTStatementValue::FuncDef(f) => { self.build_funcdef(bt,f)?; },
            PTStatementValue::ProcDef(p) => { self.build_procdef(bt,p)?; }
            PTStatementValue::Code(c) => { self.define_code(c,bt)?; },

            // XXX declare after eval
            PTStatementValue::LetStatement(vv,xx) => {
                self.make_statement(vv,xx,true,bt)?;
            },
            PTStatementValue::ModifyStatement(vv,xx) => {
                /* Dress up our variable, temporarily */
                let vv = vv.iter().map(|v| PTLetAssign::Variable(v.clone(),vec![])).collect::<Vec<_>>();
                let xx = xx.iter().map(|x| OrBundle::Normal(x.clone())).collect::<Vec<_>>();
                self.make_statement(&vv,&xx,false,bt)?;
            },
            PTStatementValue::BareCall(c) => {
                self.build_proc(None,bt,c)?;
            },
        }
        Ok(())
    }
}
