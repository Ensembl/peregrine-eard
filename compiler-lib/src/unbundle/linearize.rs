use std::{collections::{HashMap}};
use crate::{frontend::{femodel::{OrBundle, OrBundleRepeater}, buildtree::Variable}, model::{linear::{LinearStatementValue, LinearStatement}, checkstypes::{TypedArgument}, compiled::{Metadata, ProgramName}}, controller::{source::ParsePosition, serialise::OpcodeVersion}};
use crate::frontend::{buildtree::{BuildTree, BTStatement, BTStatementValue, BTLValue, BTProcCall, BTExpression, BTRegisterType, BTFuncProcDefinition, BTTopDefn}};
use super::{unbundleaux::{Position, VarRegisters, Transits, Checks}, repeater::{find_repeater_arguments, rewrite_repeater}};

/* NOTE! After linearizing we are not yet in signle-assignment form as multiple consecutive calls
 * to a function/procedure reuse registers (we can get away without rewriting or a stack because)
 * we are non-recursive. It takes a call to reduce to convert to single-assignment form. This is
 * also why reduce() is not an EquivelenceClass as the semantics are slightly different. (We are
 * also lazy during sequence generation and reuse registers but this could be changed were it
 * necessary that we are not in SAF anyway).
 */

pub(crate) struct Allocator {
    next_register: usize,
    next_call_index: usize
}

impl Allocator {
    pub(crate) fn next_register(&mut self) -> usize {
        self.next_register += 1;
        self.next_register
    }

    pub(crate) fn next_call(&mut self) -> usize {
        self.next_call_index += 1;
        self.next_call_index
    }

    pub(crate) fn verbose(&self) {
        eprintln!("used {} registers; made {} calls",self.next_register,self.next_call_index);
    }
}

struct Linearize<'a> {
    tree: &'a BuildTree,
    bundles: &'a Transits,
    output: Vec<LinearStatement>,
    positions: ParsePosition,
    var_registers: VarRegisters,
    bt_registers: HashMap<(usize,Option<String>),usize>,
    call_stack: Vec<usize>,
    captures: HashMap<usize,Vec<(Variable,usize)>>,
    checks: Checks,
    allocator: Allocator,
    name: Option<ProgramName>,
    version: OpcodeVersion
}

impl<'a> Linearize<'a> {
    fn new(tree: &'a BuildTree, bundles: &'a Transits) -> Linearize<'a> {
        Linearize {
            tree, bundles, 
            output: vec![], 
            positions: ParsePosition::empty("called"),
            var_registers: VarRegisters::new(),
            bt_registers: HashMap::new(),
            allocator: Allocator {
                next_register: 0,
                next_call_index: 0
            },
            call_stack: vec![],
            captures: HashMap::new(),
            checks: Checks::new(),
            name: None,
            version: OpcodeVersion::new()
        }
    }

    fn bt_register(&mut self, reg: usize, name: Option<&String>) -> usize {
        let (registers,allocator) = (&mut self.bt_registers,&mut self.allocator);
        *registers.entry((reg,name.cloned())).or_insert_with(|| {
            allocator.next_register()
        })
    }

    fn add(&mut self, value: LinearStatementValue) {
        self.output.push(LinearStatement {
            value,
            position: self.positions.clone()
        })
    }

    fn callee_args(&mut self, defn_args: &[OrBundle<TypedArgument>], arg_regs: &[usize]) -> Result<(),String> {
        let mut regs = arg_regs.iter();
        for (i,arg) in defn_args.iter().enumerate() {
            match arg {
                OrBundle::Normal(arg) => {
                    let var_reg = self.allocator.next_register();
                    self.var_registers.add(&Variable { name: arg.id.clone(), prefix: None },var_reg);
                    self.add(LinearStatementValue::Copy(var_reg,*regs.next().unwrap()));
                    if arg.typespec.arg_types.len() > 0 {
                        self.add(LinearStatementValue::Signature(vec![(var_reg,arg.typespec.arg_types.clone())]));
                    }
                    for check in &arg.typespec.checks {
                        let check_index = self.checks.get(&check.check_type,&check.name);
                        self.add(LinearStatementValue::Check(check.name.clone(),var_reg,check.check_type.clone(),check_index,check.force));
                    }
                },
                OrBundle::Bundle(bundle_name) => {
                    let bundle = self.bundles.get(&self.call_stack,&Position::Arg(i))?;
                    for name in bundle {
                        let var_reg = self.allocator.next_register();
                        self.var_registers.add(&Variable { name: name.clone(), prefix: Some(bundle_name.clone()) },var_reg);
                        self.add(LinearStatementValue::Copy(var_reg,*regs.next().unwrap()));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_arg_match(&self, defn: &BTFuncProcDefinition, args: &[OrBundleRepeater<BTExpression>]) -> Result<(),String> {
        if defn.args.len() != args.len() {
            return Err(format!("definition at {} has {} args; call passes {}",defn.position.full_str(),defn.args.len(),args.len()));
        }
        Ok(())
    }

    fn check_ret_match(&self, defn: &BTFuncProcDefinition, rets: &Option<Vec<OrBundleRepeater<BTLValue>>>) -> Result<(),String> {
        if let Some(rets) = rets {
            if defn.ret.len() != rets.len() {
                return Err(format!("definition at {} has {} return values; call expects {}",defn.position.full_str(),defn.ret.len(),rets.len()));
            }    
        }
        Ok(())
    }

    fn ret_checks(&mut self, index: usize, reg: usize, defn: &BTFuncProcDefinition) -> Result<(),String> {
        if let Some(type_spec) = defn.ret_type.as_ref().and_then(|v| v.get(index)) {
            if type_spec.arg_types.len() > 0 {
                self.add(LinearStatementValue::Signature(vec![(reg,type_spec.arg_types.clone())]));
            }
            for check in &type_spec.checks {
                let check_index = self.checks.get(&check.check_type,&check.name);
                self.add(LinearStatementValue::Check(check.name.clone(),reg,check.check_type.clone(),check_index,check.force));
            }
        }
        Ok(())
    }

    fn callee_rets(&mut self, defn: &BTFuncProcDefinition) -> Result<Vec<usize>,String> {
        let mut regs = vec![];
        for (i,ret) in defn.ret.iter().enumerate() {
            match ret {
                OrBundle::Normal(expr) => {
                    match expr {
                        BTExpression::Constant(c) => {
                            let reg = self.allocator.next_register();
                            regs.push(reg);
                            self.add(LinearStatementValue::Constant(reg,c.clone()));
                            self.ret_checks(i,reg,defn)?;
                        },
                        BTExpression::Variable(variable) => {
                            let reg = self.var_registers.get(variable)?;
                            regs.push(reg);
                            self.ret_checks(i,reg,defn)?;
                        },
                        BTExpression::RegisterValue(src,BTRegisterType::Normal) => {
                            let var_reg = self.bt_register(*src,None);
                            regs.push(var_reg);
                            self.ret_checks(i,var_reg,defn)?;
                        },
                        BTExpression::RegisterValue(src,BTRegisterType::Bundle) => {
                            let bundle = self.bundles.get(&self.call_stack,&Position::Return(i))?;
                            for name in bundle {
                                let var_reg = self.bt_register(*src,Some(name));
                                regs.push(var_reg);
                            }        
                        },
                        BTExpression::Function(func) => {
                            self.call_stack.push(func.call_index);
                            let func_args = self.caller_args(&func.args)?;
                            let fdefn = self.tree.get_function(func)?;
                            self.check_arg_match(fdefn,&func.args)?;
                            let mut ret_regs = self.callee(func.func_index,&fdefn,&func_args)?;
                            if ret_regs.len() == 1 { // only excludes some bundles, which are untyped
                                self.ret_checks(i,ret_regs[0],defn)?;
                            }
                            regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundle::Bundle(prefix) => {
                    if self.var_registers.check_used(prefix) {
                        let bundle = self.bundles.get(&self.call_stack,&Position::Return(i))?;
                        for name in bundle {
                            let var_reg = self.var_registers.get(&Variable { name: name.clone(), prefix: Some(prefix.clone()) })?;
                            regs.push(var_reg);
                        }
                    }
                }
            }
        } 
        Ok(regs)
    }

    fn callee_captures(&mut self, index: usize) -> Result<(),String> {
        if let Some(captures) = self.captures.get(&index).cloned() {
            for (variable, reg) in captures {
                let arg = self.allocator.next_register();
                self.add(LinearStatementValue::Copy(arg,reg));
                self.var_registers.add(&variable,arg);
            }
        }
        Ok(())
    }

    fn callee_matches(&mut self, defn: &BTFuncProcDefinition, arg_regs: &[usize], ret_regs: &[usize]) -> Result<(),String> {
        let mut regs = vec![];
        for (defn,reg) in defn.args.iter().zip(arg_regs.iter()) {
            match defn {
                OrBundle::Normal(defn) => {
                    regs.push((*reg,defn.typespec.arg_types.clone()));
                }
                _ => {}
            }
        }
        if let Some(ret_type) = &defn.ret_type {
            for (defn,reg) in ret_type.iter().zip(ret_regs.iter()) {
                regs.push((*reg,defn.arg_types.clone()));
            }
        }
        let regs = regs.iter().filter(|(_,t)| t.len()>0).cloned().collect::<Vec<_>>();
        if regs.len() > 0 {
            self.add(LinearStatementValue::Signature(regs));
        }
        Ok(())
    }

    fn callee(&mut self, index: usize, defn: &BTFuncProcDefinition, arg_regs: &[usize]) -> Result<Vec<usize>,String> {
        self.var_registers.push();
        self.checks.push();
        let old_pos = self.positions.clone();
        self.positions = self.positions.add(defn.position.last());
        self.callee_args(&defn.args,arg_regs)?;
        self.callee_captures(index)?;
        for stmt in &defn.block {
            self.statement(stmt)?;
        }
        let callee_rets = self.callee_rets(&defn)?;
        self.callee_matches(defn,&arg_regs,&callee_rets)?;
        self.positions = old_pos;
        self.checks.pop();
        self.var_registers.pop();
        Ok(callee_rets)
    }

    fn caller_args(&mut self, args: &[OrBundleRepeater<BTExpression>]) -> Result<Vec<usize>,String> {
        let mut arg_regs = vec![];
        for (i,arg) in args.iter().enumerate() {
            match arg {
                OrBundleRepeater::Normal(expr) => {
                    match expr {
                        BTExpression::Constant(c) => { 
                            let arg_reg = self.allocator.next_register();
                            self.add(LinearStatementValue::Constant(arg_reg,c.clone()));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::Variable(variable) => {
                            let arg_reg = self.allocator.next_register();
                            let src_reg = self.var_registers.get(variable)?;
                            self.add(LinearStatementValue::Copy(arg_reg,src_reg));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::RegisterValue(bt_reg,BTRegisterType::Normal) => { 
                            let arg_reg = self.allocator.next_register();
                            let src_reg = self.bt_register(*bt_reg,None);
                            self.add(LinearStatementValue::Copy(arg_reg,src_reg));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::RegisterValue(bt_register,BTRegisterType::Bundle) => {
                            let bundle = self.bundles.get(&self.call_stack,&Position::Return(i))?;
                            for bundle_arg in bundle {
                                let arg_reg = self.allocator.next_register();
                                let reg = self.bt_register(*bt_register,Some(bundle_arg));
                                self.add(LinearStatementValue::Copy(arg_reg,reg));
                                arg_regs.push(arg_reg);
                            }                                    
                        },
                        BTExpression::Function(func) => {
                            self.call_stack.push(func.call_index);
                            let func_args = self.caller_args(&func.args)?;
                            let defn = self.tree.get_function(func)?;
                            self.check_arg_match(defn,&func.args)?;
                            let mut ret_regs = self.callee(func.func_index,&defn,&func_args)?;
                            arg_regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundleRepeater::Bundle(bundle_name) =>  {
                    if self.var_registers.check_used(bundle_name) {
                        let bundle = self.bundles.get(&self.call_stack,&Position::Arg(i))?;
                        for bundle_arg in bundle {
                            let arg_reg = self.allocator.next_register();
                            let variable = self.var_registers.get(&Variable {
                                prefix: Some(bundle_name.to_string()),
                                name: bundle_arg.to_string()
                            })?;
                            self.add(LinearStatementValue::Copy(arg_reg,variable));
                            arg_regs.push(arg_reg);
                        }
                    }
                },
                OrBundleRepeater::Repeater(_) => { panic!("Should be no repeaters here/A") },
            }
        }
        Ok(arg_regs)
    }

    fn callee_to_caller(&mut self, rets: &[OrBundleRepeater<BTLValue>], callee_rets: &[usize]) -> Result<(),String> {
        let mut src = callee_rets.iter();
        for (i,ret) in rets.iter().enumerate() {
            match ret {
                OrBundleRepeater::Normal(BTLValue::Variable(variable)) => {
                    let dst = self.allocator.next_register();
                    self.var_registers.add(&variable,dst);
                    self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                },
                OrBundleRepeater::Normal(BTLValue::Register(register,BTRegisterType::Normal)) => {
                    let dst = self.bt_register(*register,None);
                    self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                },
                OrBundleRepeater::Normal(BTLValue::Register(bt_register,BTRegisterType::Bundle)) => {
                    let bundle = self.bundles.get(&self.call_stack,&Position::Return(i))?;
                    for bundle_arg in bundle {
                        let dst = self.bt_register(*bt_register,Some(bundle_arg));
                        self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                    }    
                },
                OrBundleRepeater::Bundle(bundle_name) => {
                    let bundle = self.bundles.get(&self.call_stack,&Position::Return(i))?;
                    for bundle_arg in bundle {
                        let dst = self.allocator.next_register();
                        self.var_registers.add(&Variable {
                            prefix: Some(bundle_name.to_string()),
                            name: bundle_arg.to_string()
                        },dst);
                        self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                    }
                },
                OrBundleRepeater::Repeater(_) => { panic!("got repeater after elimination"); }
            };
        }
        Ok(())
    }

    fn non_repeater_procedure(&mut self, proc: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        let arg_regs = self.caller_args(&proc.args)?;
        match self.tree.get_any(proc)? {
            Some(BTTopDefn::FuncProc(defn)) => {
                /* normal procedure */
                if defn.entry {
                    return Err("entry procedures cannot be explicitly called".to_string());
                }
                self.check_arg_match(defn,&proc.args)?;
                self.check_ret_match(defn,&proc.rets)?;
                let callee_rets = self.callee(proc.proc_index.unwrap(),defn,&arg_regs)?;
                self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&callee_rets)?;
            },
            Some(BTTopDefn::Code(defn)) => {
                /* code */
                let callee_rets = (0..defn.ret_count()).map(|_| self.allocator.next_register()).collect::<Vec<_>>();
                let call_index = self.allocator.next_call();
                self.add(LinearStatementValue::Code(
                    call_index,
                    proc.proc_index.expect("assignment/non-assignment"),
                    callee_rets.clone(),arg_regs
                ));
                self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&callee_rets)?;
            },
            None => {
                /* assignment */
                self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&arg_regs)?;
            }
        }
        Ok(())
    }

    fn procedure(&mut self, proc: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        self.call_stack.push(proc.call_index);
        if let Some((left_prefix,right_prefix)) = find_repeater_arguments(proc)? {
            if self.var_registers.check_used(&right_prefix) {
                for name in self.bundles.get(&self.call_stack,&Position::Repeater)? {
                    let left = Variable { prefix: Some(left_prefix.clone()), name: name.to_string() };
                    let right = Variable { prefix: Some(right_prefix.clone()), name: name.to_string() };
                    let call = rewrite_repeater(proc,&left,&right)?;
                    self.non_repeater_procedure(&call)?;
                }
            }
        } else {
            self.non_repeater_procedure(&proc)?;
        }
        self.call_stack.pop();
        Ok(())
    }

    fn define(&mut self, index: usize) -> Result<(),String> {
        match self.tree.get_by_index(index)? {
            BTTopDefn::FuncProc(defn) => {
                let mut captures = vec![];                
                for capture in defn.captures.iter() {
                    match capture {
                        OrBundle::Normal(variable) => {
                            let dst = self.allocator.next_register();
                            captures.push((variable.clone(),dst));
                            let src = self.var_registers.get(variable)?;
                            self.add(LinearStatementValue::Copy(dst,src));
                        },
                        OrBundle::Bundle(prefix) => {
                            for name in self.var_registers.outer_all_prefix(prefix) {
                                let dst = self.allocator.next_register();
                                let variable = Variable {
                                    prefix: Some(prefix.to_string()),
                                    name: name.clone()
                                };
                                captures.push((variable.clone(),dst));
                                let src = self.var_registers.get_outer(&variable)?;
                                self.add(LinearStatementValue::Copy(dst,src));
                            }
                        },
                    }
                }
                self.captures.insert(index,captures);
            },
            BTTopDefn::Code(_) => {}
        }
        Ok(())
    }

    fn statement(&mut self, stmt: &BTStatement) -> Result<(),String> {
        self.positions.update(stmt.position.last());
        match &stmt.value {
            BTStatementValue::Define(index) => {
                self.define(*index)?;
            },
            BTStatementValue::Declare(_) => {},
            BTStatementValue::Check(variable,check) => {
                let register = self.var_registers.get(variable)?;
                let check_index = self.checks.get(&check.check_type,&check.name);
                self.add(LinearStatementValue::Check(check.name.clone(),register,check.check_type.clone(),check_index,check.force));
            },
            BTStatementValue::BundledStatement(proc) => {
                self.procedure(proc)?;
            },
            BTStatementValue::Entry(idx,name) => {
                self.add(LinearStatementValue::Entry(name.to_string()));
                match self.tree.get_by_index(*idx)? {
                    BTTopDefn::FuncProc(defn) => {
                        self.callee(*idx,defn,&[])?;
                    },
                    BTTopDefn::Code(_) => {
                        return Err("cannot enter at code section!".to_string())
                    }
                }
            },
            BTStatementValue::Header(group,name,version) => {
                if self.output.len() > 0 {
                    return Err("program headr must be first statement".to_string());
                }
                self.name = Some(ProgramName {
                    group: group.to_string(),
                    name: name.to_string(),
                    version: *version
                });
            },
            BTStatementValue::Version(name,a,b) => {
                self.version.add(name,(*a,*b));
            },
        }
        Ok(())
    }
}

pub(crate) fn linearize(tree: &BuildTree, bundles: &Transits,verbose: bool) -> Result<(Vec<LinearStatement>,Allocator,Metadata),String> {
    let mut linearize = Linearize::new(tree,bundles);
    for stmt in &tree.statements {
        linearize.statement(stmt).map_err(|e| linearize.positions.message(&e))?;
    }
    if linearize.name.is_none() {
        return Err("missing program header".to_string());
    }
    if verbose {
        eprintln!("linearised to {} statements",linearize.output.len());
        linearize.allocator.verbose();
    }
    let metadata = Metadata { name: linearize.name.unwrap(), version: linearize.version };
    Ok((linearize.output,linearize.allocator,metadata))
}
