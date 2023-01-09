use std::{collections::{BTreeMap, HashSet}};
use crate::{model::{checkstypes::{TypedArgument, ArgTypeSpec, TypeSpec, Check}, codeblocks::{CodeDefinition, CodeBlock}}, controller::source::ParsePosition};
use super::{buildtree::{BuildTree, BTStatementValue, BTStatement, BTExpression, BTDefinitionVariety, BTFuncProcDefinition, BTDefinition, BTFuncCall, BTLValue, BTProcCall, BTRegisterType, Variable}, parsetree::{PTExpression, PTCall, PTStatement, PTStatementValue, PTFuncDef, PTProcDef, FuncProcModifier}, femodel::{OrBundle, OrBundleRepeater}};

#[derive(Debug,Clone)]
pub(super) enum DefName {
    Func(usize),
    Proc(usize),
    Code(usize)
}

struct CurrentFuncProcDefinition {
    position: ParsePosition,
    block: Vec<BTStatement>,
    name: String,
    export: bool,
    entry: bool,
    versions: Vec<Vec<String>>,
    args: Vec<OrBundle<TypedArgument>>,
    captures: Vec<OrBundle<Variable>>,
    variety: BTDefinitionVariety,
    ret_type: Option<Vec<ArgTypeSpec>>, // exactly one for functions
}

impl CurrentFuncProcDefinition {
    /* only one wild per arg_spec and if seqwild then no other seq */
    fn verify_argret(&self, spec: &[TypeSpec]) -> Result<(),String> {
        let mut wild = 0;
        let mut atomic = HashSet::new();
        let mut sequence = HashSet::new();
        for argtype in spec {
            match argtype {
                TypeSpec::Atomic(a) => { atomic.insert(a.clone()); },
                TypeSpec::Sequence(a) => { sequence.insert(a.clone()); },
                TypeSpec::Wildcard(_) => { wild += 1; },
                TypeSpec::AtomWildcard(_) => { wild += 1; },
                TypeSpec::SequenceWildcard(_) => { wild += 1; },
            }
        }
        if wild > 1 {
            return Err(format!("only one wildcard allowed per argument"));
        }
        if wild > 0 && atomic.len()+sequence.len() > 0 {
            return Err(format!("cannot mix wild and non-wild types in argument"));
        }
        if atomic != sequence && atomic.len() > 0 && sequence.len() > 0 {
            return Err(format!("sequence types must match non-sequence types in single signature"));
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
            block: self.block.clone(),
            entry: self.entry
        })
    }
}

pub(crate) struct BuildContext {
    target_version: Option<u32>,
    location: ParsePosition,
    file_context: usize,
    defnames: BTreeMap<(Option<usize>,String),DefName>,
    next_register: usize,
    funcproc_target: Option<CurrentFuncProcDefinition>,
    next_call_index: usize
}

impl BuildContext {
    pub(crate) fn new(target_version: Option<u32>) -> BuildContext {
        BuildContext {
            target_version,
            location: ParsePosition::empty("included"),
            file_context: 0,
            defnames: BTreeMap::new(),
            next_register: 0,
            funcproc_target: None,
            next_call_index: 0
        }
    }

    pub(crate) fn push_funcproc_target(&mut self, is_proc: bool, name: &str,
            args: &[OrBundle<TypedArgument>],
            ret_type: Option<Vec<ArgTypeSpec>>,
            captures: &[OrBundle<Variable>],
            export: bool, entry: bool, versions: Vec<Vec<String>>) {
        let variety = if is_proc { BTDefinitionVariety::Proc } else { BTDefinitionVariety::Func };
        self.funcproc_target = Some(CurrentFuncProcDefinition {
            position: self.location.clone(),
            name: name.to_string(),
            args: args.to_vec(),
            captures: captures.to_vec(),
            export, entry, variety,
            block: vec![],
            ret_type, versions
        });
    }

    fn version_part_ok(&self, defn_ver: &str) -> bool {
        let lt = defn_ver.contains("<");
        let gt = defn_ver.contains(">");
        let eq = defn_ver.contains("=");
        let digits = defn_ver.chars().filter(|x| x.is_digit(10)).collect::<String>();
        digits.parse::<u32>().map(|v| {
            if let Some(want_ver) = self.target_version {
                ( gt && want_ver > v ) || ( eq && want_ver == v ) || ( lt && want_ver < v )
            } else { gt }
        }).unwrap_or(false)
    }

    fn version_ok(&self, defn_ver: &Vec<String>) -> bool {
        defn_ver.iter().all(|spec| self.version_part_ok(spec))
    }

    fn versions_ok(&self, defn_vers: &Vec<Vec<String>>) -> bool {
        if defn_vers.len() == 0 { return true; }
        defn_vers.iter().any(|spec| self.version_ok(spec))
    }

    pub(crate) fn pop_funcproc_target(&mut self, ret: &[OrBundle<BTExpression>], bt: &mut BuildTree) -> Result<(),String> {
        let ctx = self.funcproc_target.take().expect("pop without push");
        if !self.versions_ok(&ctx.versions) { return Ok(()); }
        let defn_id = match &ctx.variety {
            BTDefinitionVariety::Func => {                
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret)?,false,bt,ctx.export)?
            },
            BTDefinitionVariety::Proc => {
                self.define_funcproc(&ctx.name,ctx.to_funcproc(ret)?,true,bt,ctx.export)?
            }
        };
        self.add_statement(bt,BTStatementValue::Define(defn_id))?;
        if ctx.entry {
            bt.set_entry(defn_id,&ctx.name);
            if ctx.args.len() > 0 {
                return Err(format!("entry procedure {} can take no arguments",ctx.name));
            }
            if ret.len() > 0 {
                return Err(format!("entry procedure {} can return no values",ctx.name));
            }
            if ctx.captures.len() > 0 {
                return Err(format!("entry procedure {} cannot capture values",ctx.name));
            }
        }
        Ok(())
    }

    pub(crate) fn set_file_context(&mut self, context: usize) {
        self.file_context = context;
    }

    pub(crate) fn set_location(&mut self, position: &ParsePosition) {
        self.location = position.clone();
    }

    pub(crate) fn location(&self) -> &ParsePosition { &self.location }

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

    pub(super) fn define_funcproc(&mut self, name: &str, fpdefn: BTFuncProcDefinition, is_proc: bool, bt: &mut BuildTree, export: bool) -> Result<usize,String> {
        let definition = if is_proc { BTDefinition::Proc(fpdefn) } else { BTDefinition::Func(fpdefn) };
        let id = bt.add_definition(definition);
        let name_id = if is_proc { DefName::Proc(id) } else { DefName::Func(id) };
        let context = if export { None } else { Some(self.file_context) };
        let key = (context,name.to_string());
        if self.defnames.contains_key(&key) {
            return Err(format!("duplicate definition for {}",name));
        }
        self.defnames.insert(key,name_id);
        Ok(id)
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
            position: self.location.clone()
        };
        if let Some(target) = self.funcproc_target.as_mut() {
            target.block.push(stmt);
        } else {
            bt.add_statement(stmt);
        }
        Ok(())
    }

    fn is_top(&self, x: &OrBundle<PTExpression>) -> Result<bool,String> {
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
        if xx.len() == 1 && self.is_top(&xx[0])? {
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
        let versions = def.versions();
        self.push_funcproc_target(false,&def.name,&def.args,ret_type,&def.captures,export,false,versions);
        for stmt in &def.block {
            self.build_statement(bt,stmt)?;
        }
        let expr = self.build_expr_ob(bt,&def.value)?;
        self.pop_funcproc_target(&[expr],bt)?;
        Ok(())
    }

    fn build_procdef(&mut self, bt: &mut BuildTree, def: &PTProcDef) -> Result<(),String> {
        let versions = def.versions();
        let export = def.modifiers.contains(&FuncProcModifier::Export);
        let entry = def.modifiers.contains(&FuncProcModifier::Entry);
        self.push_funcproc_target(true,&def.name,&def.args,def.ret_type.clone(),&def.captures,export,entry,versions);
        for stmt in &def.block {
            self.build_statement(bt,stmt)?;
        }
        let ret = def.ret.iter().map(|x| self.build_expr_ob(bt,x)).collect::<Result<Vec<_>,_>>()?;
        self.pop_funcproc_target(&ret,bt)?;
        Ok(())
    }

    pub(super) fn build_statement(&mut self, bt: &mut BuildTree, stmt: &PTStatement) -> Result<(),String> {
        match &stmt.value {
            PTStatementValue::Include(_,_) | PTStatementValue::MacroCall(_) => {
                panic!("item should have been eliminated from build tree");
            },
            PTStatementValue::Header(group,name,version) => {
                self.add_statement(bt,BTStatementValue::Header(group.to_string(),name.to_string(),*version))?;
            },
            PTStatementValue::Version(name,a,b) => {
                self.add_statement(bt,BTStatementValue::Version(name.to_string(),*a,*b))?;
            }
            PTStatementValue::FuncDef(f) => { self.build_funcdef(bt,f)?; },
            PTStatementValue::ProcDef(p) => { self.build_procdef(bt,p)?; },
            PTStatementValue::Code(c) => { self.define_code(c,bt)?; },

            PTStatementValue::LetStatement(vv,xx) => {
                self.make_statement(vv,xx,true,bt)?;
            },
            PTStatementValue::LetRepeaterStatement(a,b) => {
                self.add_statement(bt,BTStatementValue::Declare(OrBundleRepeater::Repeater(a.to_string())))?;
                let stmt = self.make_statement_value(None,vec![
                        OrBundleRepeater::Repeater(b.to_string())
                    ],Some(vec![
                        OrBundleRepeater::Repeater(a.to_string())
                    ]
                ));
                self.add_statement(bt,stmt)?;
            },
            PTStatementValue::ModifyStatement(vv,xx) => {
                /* Dress up our variable, temporarily */
                let vv = vv.iter().map(|v| OrBundleRepeater::Normal((v.clone(),vec![]))).collect::<Vec<_>>();
                let xx = xx.iter().map(|x| OrBundle::Normal(x.clone())).collect::<Vec<_>>();
                self.make_statement(&vv,&xx,false,bt)?;
            },
            PTStatementValue::Expression(expr) => {
                if let PTExpression::Call(c) = expr {
                    match self.lookup(&c.name)? {
                        DefName::Func(_) => {},
                        DefName::Proc(_) => { return self.build_top(None,bt,c); },
                        DefName::Code(_) => { return self.build_top(None,bt,c); },
                    }
    
                }
                /* top level is function call with discarded result, add assign proc */
                let ret = OrBundleRepeater::Normal(self.build_expression(bt,&expr)?);
                let stmt = self.make_statement_value(None,vec![ret],None);
                self.add_statement(bt,stmt)?;
            },
        }
        Ok(())
    }
}
