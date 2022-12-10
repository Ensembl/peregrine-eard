use std::{collections::{HashMap}, sync::Arc};
use crate::{buildtree::{BuildTree, BTStatement, BTStatementValue, BTLValue, BTProcCall, BTExpression, BTRegisterType, BTFuncProcDefinition}, parsetree::at, model::{Variable, OrBundleRepeater, LinearStatement, LinearStatementValue, OrBundle, TypedArgument}};
use super::{unbundleaux::{Position, VarRegisters}, repeater::{find_repeater_arguments, rewrite_repeater}};

type Bundles = HashMap<(Vec<usize>,Position),Vec<String>>;

struct Linearize<'a> {
    tree: &'a BuildTree,
    bundles: &'a Bundles,
    output: Vec<LinearStatement>,
    positions: Vec<(Arc<Vec<String>>,usize)>,
    next_register: usize,
    var_registers: VarRegisters,
    bt_registers: HashMap<(usize,Option<String>),usize>,
    call_stack: Vec<usize>
}

impl<'a> Linearize<'a> {
    fn new(tree: &'a BuildTree, bundles: &'a Bundles) -> Linearize<'a> {
        Linearize {
            tree, bundles, 
            output: vec![], 
            positions: vec![],
            var_registers: VarRegisters::new(),
            bt_registers: HashMap::new(),
            next_register: 0,
            call_stack: vec![]
        }
    }

    fn anon_register(&mut self) -> usize {
        self.next_register += 1;
        self.next_register
    }

    fn bt_register(&mut self, reg: usize, name: Option<&String>) -> usize {
        let (registers,next_register) = (&mut self.bt_registers,&mut self.next_register);
        *registers.entry((reg,name.cloned())).or_insert_with(|| {
            *next_register += 1;
            *next_register
        })
    }

    fn add(&mut self, value: LinearStatementValue) {
        let position = self.positions.last().cloned().unwrap_or_else(|| (Arc::new(vec!["*anon*".to_string()]),0));
        self.output.push(LinearStatement {
            value,
            file: position.0,
            line_no: position.1
        })
    }

    fn error_at(&self, msg: &str) -> String {
        self.positions.last().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
    }

    fn callee_args(&mut self, defn_args: &[OrBundle<TypedArgument>], arg_regs: &[usize]) -> Result<(),String> {
        let mut regs = arg_regs.iter();
        for (i,arg) in defn_args.iter().enumerate() {
            match arg {
                OrBundle::Normal(arg) => {
                    let var_reg = self.anon_register();
                    self.var_registers.add(&Variable { name: arg.id.clone(), prefix: None },var_reg);
                    self.add(LinearStatementValue::Copy(var_reg,*regs.next().unwrap()));
                },
                OrBundle::Bundle(bundle_name) => {
                    let key = &(self.call_stack.clone(),Position::Arg(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/A")?;
                    for name in bundle {
                        let var_reg = self.anon_register();
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
            return Err(format!("definition at {} has {} args; call passes {}",defn.at(),defn.args.len(),args.len()));
        }
        Ok(())
    }

    fn callee_rets(&mut self, defn_ret: &[OrBundle<BTExpression>]) -> Result<Vec<usize>,String> {
        let mut regs = vec![];
        for (i,ret) in defn_ret.iter().enumerate() {
            match ret {
                OrBundle::Normal(expr) => {
                    match expr {
                        BTExpression::Constant(c) => {
                            let reg = self.anon_register();
                            regs.push(reg);
                            self.add(LinearStatementValue::Constant(reg,c.clone()));
                        },
                        BTExpression::Variable(variable) => {
                            regs.push(self.var_registers.get(variable)?);
                        },
                        BTExpression::RegisterValue(src,BTRegisterType::Normal) => {
                            regs.push(*src);
                        },
                        BTExpression::RegisterValue(src,BTRegisterType::Bundle) => {
                            let key = &(self.call_stack.clone(),Position::Return(i));
                            let bundle = self.bundles.get(&key)
                                .ok_or_else(|| "cannot find bundle/F")?;
                            for name in bundle {
                                let var_reg = self.bt_register(*src,Some(name));
                                regs.push(var_reg);
                            }        
                        },
                        BTExpression::Function(func) => {
                            self.call_stack.push(func.call_index);
                            let func_args = self.caller_args(&func.args)?;
                            let defn = self.tree.get_function(func)?;
                            self.check_arg_match(defn,&func.args)?;
                            let mut ret_regs = self.callee(&defn,&func_args)?;
                            regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundle::Bundle(prefix) => {
                    if !self.var_registers.check_used(prefix) {
                        return Err(format!("empty bundle return '{}'",prefix));
                    }
                    let key = &(self.call_stack.clone(),Position::Return(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/G")?;
                    for name in bundle {
                        let var_reg = self.var_registers.get(&Variable { name: name.clone(), prefix: Some(prefix.clone()) })?;
                        regs.push(var_reg);
                    }
                }
            }
        } 
        Ok(regs)
    }

    // TODO type checking!
    fn callee(&mut self, defn: &BTFuncProcDefinition, arg_regs: &[usize]) -> Result<Vec<usize>,String> {
        self.var_registers.push();
        self.callee_args(&defn.args,arg_regs)?;
        self.positions.push(defn.position.clone());
        for stmt in &defn.block {
            self.statement(stmt)?;
        }
        let callee_rets = self.callee_rets(&defn.ret)?;
        self.positions.pop();
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
                            let arg_reg = self.anon_register();
                            self.add(LinearStatementValue::Constant(arg_reg,c.clone()));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::Variable(variable) => {
                            let arg_reg = self.anon_register();
                            let src_reg = self.var_registers.get(variable)?;
                            self.add(LinearStatementValue::Copy(arg_reg,src_reg));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::RegisterValue(bt_reg,BTRegisterType::Normal) => { 
                            let arg_reg = self.anon_register();
                            let src_reg = self.bt_register(*bt_reg,None);
                            self.add(LinearStatementValue::Copy(arg_reg,src_reg));
                            arg_regs.push(arg_reg);
                        },
                        BTExpression::RegisterValue(bt_register,BTRegisterType::Bundle) => {
                            let key = &(self.call_stack.clone(),Position::Return(i));
                            let bundle = self.bundles.get(&key)
                                .ok_or_else(|| "cannot find bundle/B")?;
                            for bundle_arg in bundle {
                                let arg_reg = self.anon_register();
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
                            let mut ret_regs = self.callee(&defn,&func_args)?;
                            arg_regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundleRepeater::Bundle(bundle_name) =>  {
                    if !self.var_registers.check_used(bundle_name) {
                        return Err(format!("empty bundle argument '{}'",bundle_name));
                    }
                    let key = &(self.call_stack.clone(),Position::Arg(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/C")?;
                    for bundle_arg in bundle {
                        let arg_reg = self.anon_register();
                        let variable = self.var_registers.get(&Variable {
                            prefix: Some(bundle_name.to_string()),
                            name: bundle_arg.to_string()
                        })?;
                        self.add(LinearStatementValue::Copy(arg_reg,variable));
                        arg_regs.push(arg_reg);
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
                    let dst = self.anon_register();
                    self.var_registers.add(&variable,dst);
                    self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                },
                OrBundleRepeater::Normal(BTLValue::Register(register,BTRegisterType::Normal)) => {
                    let dst = self.bt_register(*register,None);
                    self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                },
                OrBundleRepeater::Normal(BTLValue::Register(bt_register,BTRegisterType::Bundle)) => {
                    let key = &(self.call_stack.clone(),Position::Return(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/D")?;
                    for bundle_arg in bundle {
                        let dst = self.bt_register(*bt_register,Some(bundle_arg));
                        self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                    }    
                },
                OrBundleRepeater::Bundle(bundle_name) => {
                    let key = &(self.call_stack.clone(),Position::Return(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/E")?;
                    for bundle_arg in bundle {
                        let dst = self.anon_register();
                        self.var_registers.add(&Variable {
                            prefix: Some(bundle_name.to_string()),
                            name: bundle_arg.to_string()
                        },dst);
                        self.add(LinearStatementValue::Copy(dst,*src.next().unwrap()));
                    }
                },
                OrBundleRepeater::Repeater(_) => todo!(),
            };
        }
        Ok(())
    }

    fn procedure(&mut self, proc: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        self.call_stack.push(proc.call_index);
        if let Some((left_prefix,right_prefix)) = find_repeater_arguments(proc)? {
            if !self.var_registers.check_used(&right_prefix) {
                return Err(format!("empty bundle used in repeater '{}'",right_prefix));
            }
            /* repeaters (recurses back in) */
            let key = &(self.call_stack.clone(),Position::Repeater);
            for name in self.bundles.get(key).expect("missing repeater data") {
                let left = Variable { prefix: Some(left_prefix.clone()), name: name.to_string() };
                let right = Variable { prefix: Some(right_prefix.clone()), name: name.to_string() };
                let call = rewrite_repeater(proc,&left,&right)?;
                self.procedure(&call)?;
            }
        } else {
            /* non-repeaters and repeater instances */
            let arg_regs = self.caller_args(&proc.args)?;
            if let Some(defn) = self.tree.get_procedure(proc)? {
                /* normal procedure */
                self.check_arg_match(defn,&proc.args)?;
                let callee_rets = self.callee(defn,&arg_regs)?;
                self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&callee_rets)?;
            } else {
                /* assignment */
                self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&arg_regs)?;
            }
        }
        self.call_stack.pop();
        Ok(())
    }

    fn statement(&mut self, stmt: &BTStatement) -> Result<(),String> {
        self.positions.push((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            BTStatementValue::Define(_) => { /*todo!()*/ },
            BTStatementValue::Declare(_) => {},
            BTStatementValue::Check(variable,check) => {
                let register = self.var_registers.get(variable)?;
                self.add(LinearStatementValue::Check(register,check.clone()));
            },
            BTStatementValue::BundledStatement(proc) => {
                self.procedure(proc)?;
            },
        }
        self.positions.pop();
        Ok(())
    }
}


pub(crate) fn linearize(tree: &BuildTree, bundles: &Bundles) -> Result<Vec<LinearStatement>,String> {
    let mut linearize = Linearize::new(tree,bundles);
    for stmt in &tree.statements {
        linearize.statement(stmt).map_err(|e| linearize.error_at(&e))?;
    }
    Ok(linearize.output)
}
