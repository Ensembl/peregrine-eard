use std::{sync::Arc, collections::BTreeMap};
use crate::{model::{OrBundle, TypedArgument, ArgTypeSpec, FuncProcModifier, Variable, OrBundleRepeater, Check, OrRepeater, TypeSpec}, codeblocks::{CodeDefinition, CodeBlock}};
use super::{buildtree::{BuildTree, BTStatementValue, BTStatement, BTExpression, BTDefinitionVariety, BTFuncProcDefinition, BTDefinition, BTFuncCall, BTLValue, BTProcCall, BTRegisterType}, parsetree::{PTExpression, PTCall, PTStatement, PTStatementValue, PTFuncDef, PTProcDef}};

#[derive(Debug,Clone)]
pub(super) enum DefName {
    Func(usize),
    Proc(usize),
    Code(usize)
}

struct CurrentFuncProcDefinition {
    position: (Arc<Vec<String>>,usize),
    block: Vec<BTStatement>,
    name: String,
    export: bool,
    args: Vec<OrBundle<TypedArgument>>,
    captures: Vec<OrBundle<Variable>>,
    variety: BTDefinitionVariety,
    ret_type: Option<Vec<ArgTypeSpec>>, // exactly one for functions
}

impl CurrentFuncProcDefinition {
    /* only one wild per arg_spec and if seqwild then no other seq */
    fn verify_argret(&self, spec: &[TypeSpec]) -> Result<(),String> {
        let mut variety = [0,0,0,0];
        for argtype in spec {
            let index = match argtype {
                TypeSpec::Atomic(_) => 0,
                TypeSpec::Sequence(_) => 1,
                TypeSpec::Wildcard(_) => 2,
                TypeSpec::SequenceWildcard(_) => 3
            };
            variety[index] += 1;
        }
        if variety[2]+variety[3] > 1 {
            return Err(format!("only one wildcard allowed per argument"));
        }
        if variety[3] > 0 && variety[1] > 0 {
            return Err(format!("cannot match wild and non-wild sequences in argument"));
        }
        Ok(())
    }

    fn verify_args(&self) -> Result<(),String> {
        for arg in &self.args {
            match arg {
                OrBundle::Normal(n) => {
                    self.verify_argret(&n.typespec.arg_types)?;
                },
                OrBundle::Bundle(_) => {}
            }
        }
        Ok(())
    }

    fn verify_rets(&self) -> Result<(),String> {
        for ret in self.ret_type.as_ref().unwrap_or(&vec![]) {
            self.verify_argret(&ret.arg_types)?;
        }
        Ok(())
    }

    fn to_funcproc(&self, ret: &[OrBundle<BTExpression>]) -> Result<BTFuncProcDefinition,String> {
        self.verify_args()?;
        self.verify_rets()?;
        Ok(BTFuncProcDefinition {
            position: self.position.clone(),
            args: self.args.clone(),
            captures: self.captures.clone(),
            ret_type: self.ret_type.clone(),
            ret: ret.to_vec(),
            block: self.block.clone()
        })
    }
}

pub struct BuildContext {
    location: (Arc<Vec<String>>,usize),
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),DefName>,
    next_register: usize,
    funcproc_target: Option<CurrentFuncProcDefinition>,
    next_call_index: usize
}

impl BuildContext {
    pub fn new() -> BuildContext {
        BuildContext {
            location: (Arc::new(vec!["*anon*".to_string()]),0),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0,
            funcproc_target: None,
            next_call_index: 0
        }
    }

    pub fn push_funcproc_target(&mut self, is_proc: bool, name: &str,
            args: &[OrBundle<TypedArgument>],
            ret_type: Option<Vec<ArgTypeSpec>>,
            captures: &[OrBundle<Variable>],
            export: bool, bt: &mut BuildTree) {
        let variety = if is_proc { BTDefinitionVariety::Proc } else { BTDefinitionVariety::Func };
        self.funcproc_target = Some(CurrentFuncProcDefinition {
            position: self.location.clone(),
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
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret)?,false,bt,ctx.export)?
            },
            BTDefinitionVariety::Proc => {
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret)?,true,bt,ctx.export)?
            }
        };
        self.add_statement(bt,defn)?;
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

    pub(crate) fn lookup_top(&self, name: &str) -> Result<usize,String> {
        match self.lookup(name) {
            Ok(DefName::Proc(x)) => Ok(x),
            Ok(DefName::Code(x)) => Ok(x),
            _ => Err(format!("cannot find procedure/code {}",name))
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
            return Err(format!("duplicate definition for {}",name));
        }
        self.defnames.insert(key,name_id);
        Ok(BTStatementValue::Define(id))
    }

    pub(crate) fn define_code(&mut self, block: &CodeBlock, bt: &mut BuildTree) -> Result<(),String> {
        let key = (Some(self.file_context),block.name.to_string());
        if !self.defnames.contains_key(&key) {
            let id = bt.add_definition(BTDefinition::Code(CodeDefinition::new()));
            let name_id = DefName::Code(id);
            self.defnames.insert(key.clone(),name_id);
        }
        match self.defnames.get(&key) {
            Some(DefName::Code(id)) => {
                bt.add_code(*id,block)?;
                Ok(())
            },
            _ => {
                Err(format!("duplicate definition for {}",block.name))
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

    fn is_top(&self, x: &OrBundle<PTExpression>, bt: &mut BuildTree) -> Result<bool,String> {
        Ok(match x {
            OrBundle::Normal(PTExpression::Call(c)) => {
                match self.lookup(&c.name)? {
                    DefName::Proc(_) => true,
                    DefName::Code(_) => true,
                    DefName::Func(_) => false
                }
            },
            _ => false
        })
    }

    fn lvalue_to_rvalue(&self, lvalue: &OrBundleRepeater<BTLValue>) -> OrBundleRepeater<BTExpression> {
        lvalue.map(|x| {
            match x {
                BTLValue::Variable(v) => BTExpression::Variable(v.clone()),
                BTLValue::Register(r,t) => BTExpression::RegisterValue(*r,t.clone())
            }
        })
    }

    pub(crate) fn make_statement_value(&mut self, proc_index: Option<usize>, args: Vec<OrBundleRepeater<BTExpression>>, rets: Option<Vec<OrBundleRepeater<BTLValue>>>) -> BTStatementValue {
        self.next_call_index += 1;
        BTStatementValue::BundledStatement(BTProcCall {
            proc_index,
            args, rets,
            call_index: self.next_call_index
        })
    }

    pub(crate) fn make_function_call(&mut self, defn: usize, args: Vec<OrBundleRepeater<BTExpression>>) -> BTFuncCall {
        self.next_call_index += 1;
        BTFuncCall {
            func_index: defn,
            args,
            call_index: self.next_call_index
        }
    }

    fn make_statement(&mut self, vv: &[OrBundleRepeater<(Variable,Vec<Check>)>], xx: &[OrBundle<PTExpression>], declare: bool, bt: &mut BuildTree) -> Result<(),String> {
        /* Step 1: allocate temporaries */
        let mut regs = vec![];
        for v in vv.iter() {
            let reg = match v {
                OrBundleRepeater::Normal(_) => { 
                    OrBundleRepeater::Normal(BTLValue::Register(self.allocate_register(),BTRegisterType::Normal))
                },
                OrBundleRepeater::Bundle(_) => {
                    OrBundleRepeater::Normal(BTLValue::Register(self.allocate_register(),BTRegisterType::Bundle))
                },
                OrBundleRepeater::Repeater(r) => {
                    self.build_declare(bt,v)?; /* repeaters are used directly so need to be declared first */
                    OrBundleRepeater::Repeater(r.clone())
                }
            };
            regs.push(reg);
        }
        /* Step 2: evaluate expressions and put into temporaries */
        if xx.len() == 1 && self.is_top(&xx[0],bt)? {
            match &xx[0] {
                OrBundle::Normal(PTExpression::Call(call)) => {
                    self.build_top(Some(regs.clone()),bt,call)?;
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
                        OrBundleRepeater::Normal(expr)
                    },
                    OrBundle::Bundle(b) => {
                        OrBundleRepeater::Bundle(b.to_string())
                    }
                };
                let stmt = self.make_statement_value(None,vec![call_arg],Some(vec![reg.clone()]));
                self.add_statement(bt,stmt)?;
            }
        } else {
            return Err(format!("let tuples differ in length: {} lvalues but {} rvalues",vv.len(),xx.len()));
        }
        if declare {
            /* Step 3: declare variables */
            for v in vv.iter() {
                match v {
                    OrBundleRepeater::Repeater(_) => {},
                    _ => { self.build_declare(bt,v)? },
                }
            }
        }
        /* Step 4: assign from temporaries to variables */
        for (v,reg) in vv.iter().zip(regs.iter()) {
            let lvalue = match v {
                OrBundleRepeater::Normal((v,_)) => {
                    Some(OrBundleRepeater::Normal(BTLValue::Variable(v.clone())))
                },
                OrBundleRepeater::Bundle(b) => { 
                    Some(OrBundleRepeater::Bundle(b.to_string()))
                },
                OrBundleRepeater::Repeater(_) => { None }
            };
            if let Some(lvalue) = lvalue {
                let stmt = self.make_statement_value(None,vec![
                    self.lvalue_to_rvalue(reg)
                ],Some(vec![lvalue]));
                self.add_statement(bt,stmt)?;    
            }
        }
        /* Step 5: check sizes */
        for v in vv.iter() {
            self.build_checks(bt,v)?;
        }
        Ok(())
    }
    
    pub(crate) fn build_call(&mut self, bt: &mut BuildTree, expr: OrBundleRepeater<PTExpression>) -> Result<OrBundleRepeater<BTExpression>,String> {
        Ok(match expr {
            OrBundleRepeater::Normal(x) => OrBundleRepeater::Normal(self.build_expression(bt,&x)?),
            OrBundleRepeater::Bundle(n) => OrBundleRepeater::Bundle(n.to_string()),
            OrBundleRepeater::Repeater(n) => OrBundleRepeater::Repeater(n.to_string())
        })
    }

    fn build_top(&mut self, rets: Option<Vec<OrBundleRepeater<BTLValue>>>, bt: &mut BuildTree, call: &PTCall) -> Result<(),String> {
        let index = self.lookup_top(&call.name)?;
        let args = call.args.iter().map(|a| self.build_call(bt,a.clone())).collect::<Result<_,_>>()?;
        let stmt = self.make_statement_value(Some(index),args,rets);
        self.add_statement(bt,stmt)?;
        Ok(())
    }

    fn build_func(&mut self, bt: &mut BuildTree, call: &PTCall) -> Result<BTFuncCall,String> {
        let index = self.lookup_func(&call.name)?;
        let args = call.args.iter().map(|a| self.build_call(bt,a.clone())).collect::<Result<_,_>>()?;
        Ok(self.make_function_call(index,args))
    }

    fn build_declare(&mut self, bt: &mut BuildTree, decl: &OrBundleRepeater<(Variable,Vec<Check>)>) -> Result<(),String> {
        let decl = decl.map(|(v,_)| v.clone());
        self.add_statement(bt,BTStatementValue::Declare(decl))?;
        Ok(())
    }

    fn build_checks(&mut self, bt: &mut BuildTree, decl: &OrBundleRepeater<(Variable,Vec<Check>)>) -> Result<(),String> {
        decl.map_result::<_,_,String>(|(var,checks)| {
            for check in checks.iter() {
                let stmt = BTStatementValue::Check(var.clone(),check.clone());
                self.add_statement(bt,stmt)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn build_expression(&mut self, bt: &mut BuildTree, expr: &PTExpression) -> Result<BTExpression,String> {
        Ok(match expr {
            PTExpression::Constant(c) => BTExpression::Constant(c.clone()),

            PTExpression::FiniteSequence(items) => {
                let reg = self.allocate_register();
                let stmt = self.make_statement_value(Some(self.lookup_func("__operator_finseq")?), 
                    vec![], Some(vec![OrBundleRepeater::Normal(BTLValue::Register(reg,BTRegisterType::Normal))])); // XXX out to reg
                self.add_statement(bt,stmt)?;
                for item in items {
                    let built = self.build_expression(bt,item)?;
                    let stmt = self.make_statement_value(Some(self.lookup_top("__operator_push")?), vec![
                        OrBundleRepeater::Normal(BTExpression::RegisterValue(reg,BTRegisterType::Normal)),
                        OrBundleRepeater::Normal(built)
                    ], Some(vec![OrBundleRepeater::Normal(BTLValue::Register(reg,BTRegisterType::Normal))]));
                    self.add_statement(bt,stmt)?;
                }
                BTExpression::RegisterValue(reg,BTRegisterType::Normal)
            },
            PTExpression::InfiniteSequence(x) => {
                self.next_call_index += 1;
                BTExpression::Function(BTFuncCall {
                    func_index: self.lookup_func("__operator_infseq")?,
                    args: vec![OrBundleRepeater::Normal(self.build_expression(bt,x)?)],
                    call_index: self.next_call_index
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
            PTStatementValue::ProcDef(p) => { self.build_procdef(bt,p)?; },
            PTStatementValue::Code(c) => { self.define_code(c,bt)?; },

            PTStatementValue::LetStatement(vv,xx) => {
                self.make_statement(vv,xx,true,bt)?;
            },
            PTStatementValue::ModifyStatement(vv,xx) => {
                /* Dress up our variable, temporarily */
                let vv = vv.iter().map(|v| OrBundleRepeater::Normal((v.clone(),vec![]))).collect::<Vec<_>>();
                let xx = xx.iter().map(|x| OrBundle::Normal(x.clone())).collect::<Vec<_>>();
                self.make_statement(&vv,&xx,false,bt)?;
            },
            PTStatementValue::BareCall(c) => {
                match self.lookup(&c.name)? {
                    DefName::Func(f) => {
                        /* top level is function call with discarded result, add assign proc */
                        let expr = PTExpression::Call(c.clone());
                        let ret = OrBundleRepeater::Normal(self.build_expression(bt,&expr)?);
                        let stmt = self.make_statement_value(None,vec![ret],None);
                        self.add_statement(bt,stmt)?;
                    },
                    DefName::Proc(_) => { self.build_top(None,bt,c)? },
                    DefName::Code(_) => { self.build_top(None,bt,c)? },
                }
            },
        }
        Ok(())
    }
}
