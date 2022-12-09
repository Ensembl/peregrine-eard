use std::{collections::{HashMap, HashSet}, sync::Arc};
use crate::{buildtree::{BuildTree, BTStatement, BTStatementValue, BTLValue, BTProcCall, BTExpression, BTRegisterType, BTFuncProcDefinition}, parsetree::at, model::{Variable, OrBundleRepeater, LinearStatement, LinearStatementValue, OrBundle, TypedArgument}};
use super::unbundleaux::Position;

type Bundles = HashMap<(Vec<usize>,Position),Vec<String>>;

struct Linearize<'a> {
    tree: &'a BuildTree,
    bundles: &'a Bundles,
    output: Vec<LinearStatement>,
    positions: Vec<(Arc<Vec<String>>,usize)>,
    next_register: usize,
    var_registers: Vec<HashMap<Variable,usize>>,
    bt_registers: HashMap<(usize,Option<String>),usize>,
    call_stack: Vec<usize>
}

impl<'a> Linearize<'a> {
    fn new(tree: &'a BuildTree, bundles: &'a Bundles) -> Linearize<'a> {
        Linearize {
            tree, bundles, 
            output: vec![], 
            positions: vec![],
            var_registers: vec![HashMap::new()],
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

    fn var_register(&mut self, variable: &Variable) -> usize {
        let (registers,next_register) = (&mut self.var_registers,&mut self.next_register);
        if let Some(registers) = registers.last_mut() {
            *registers.entry(variable.clone()).or_insert_with(|| {
                *next_register += 1;
                *next_register
            })
        } else {
            panic!("empty name stack: should be impossible");
        }
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
                    let var_reg = self.var_register(&Variable { name: arg.id.clone(), prefix: None });
                    self.add(LinearStatementValue::Copy(var_reg,*regs.next().unwrap()));
                },
                OrBundle::Bundle(bundle_name) => {
                    let key = &(self.call_stack.clone(),Position::Arg(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/A")?;
                    for name in bundle {
                        let var_reg = self.var_register(&Variable { name: name.clone(), prefix: Some(bundle_name.clone()) });
                        self.add(LinearStatementValue::Copy(var_reg,*regs.next().unwrap()));
                    }
                }
            }
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
                            regs.push(self.var_register(variable));
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
                            let mut ret_regs = self.callee(&defn,&func_args)?;
                            regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundle::Bundle(prefix) => {
                    let key = &(self.call_stack.clone(),Position::Return(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/G")?;
                    for name in bundle {
                        let var_reg = self.var_register(&Variable { name: name.clone(), prefix: Some(prefix.clone()) });
                        regs.push(var_reg);
                    }
                }
            }
        } 
        Ok(regs)
    }

    // TODO catch unknown vairables in ver_variable when expected known
    // TODO type checking!
    fn callee(&mut self, defn: &BTFuncProcDefinition, arg_regs: &[usize]) -> Result<Vec<usize>,String> {
        self.var_registers.push(HashMap::new());
        self.callee_args(&defn.args,arg_regs)?;
        for stmt in &defn.block {
            self.statement(stmt)?;
        }
        let callee_rets = self.callee_rets(&defn.ret)?;
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
                            let src_reg = self.var_register(variable);
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
                            let mut ret_regs = self.callee(&defn,&func_args)?;
                            arg_regs.append(&mut ret_regs);
                            self.call_stack.pop();
                        }
                    }
                },
                OrBundleRepeater::Bundle(bundle_name) =>  {
                    let key = &(self.call_stack.clone(),Position::Arg(i));
                    let bundle = self.bundles.get(&key)
                        .ok_or_else(|| "cannot find bundle/C")?;
                    for bundle_arg in bundle {
                        let arg_reg = self.anon_register();
                        let variable = self.var_register(&Variable {
                            prefix: Some(bundle_name.to_string()),
                            name: bundle_arg.to_string()
                        });
                        self.add(LinearStatementValue::Copy(arg_reg,variable));
                        arg_regs.push(arg_reg);
                    }
                },
                OrBundleRepeater::Repeater(_) => todo!(),
            }
        }
        Ok(arg_regs)
    }

    fn callee_to_caller(&mut self, rets: &[OrBundleRepeater<BTLValue>], callee_rets: &[usize]) -> Result<(),String> {
        let mut src = callee_rets.iter();
        for (i,ret) in rets.iter().enumerate() {
            match ret {
                OrBundleRepeater::Normal(BTLValue::Variable(variable)) => {
                    let dst = self.var_register(&variable);
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
                        let variable = self.var_register(&Variable {
                            prefix: Some(bundle_name.to_string()),
                            name: bundle_arg.to_string()
                        });
                        self.add(LinearStatementValue::Copy(variable,*src.next().unwrap()));
                    }
                },
                OrBundleRepeater::Repeater(_) => todo!(),
            };
        }
        Ok(())
    }

    fn procedure(&mut self, proc: &BTProcCall<OrBundleRepeater<BTLValue>>) -> Result<(),String> {
        self.call_stack.push(proc.call_index);
        let arg_regs = self.caller_args(&proc.args)?;
        if let Some(defn) = self.tree.get_procedure(proc)? {
            let callee_rets = self.callee(defn,&arg_regs)?;
            self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&callee_rets)?;
        } else {
            self.callee_to_caller(proc.rets.as_ref().unwrap_or(&vec![]),&arg_regs)?;
        }
        self.call_stack.pop();
        Ok(())
    }

    fn statement(&mut self, stmt: &BTStatement) -> Result<(),String> {
        self.positions.push((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            BTStatementValue::Define(_) => { /*todo!()*/ },
            BTStatementValue::Declare(OrBundleRepeater::Normal(variable)) => { 
                self.var_register(variable);
            },
            BTStatementValue::Declare(_) => {},
            BTStatementValue::Check(variable,check) => {
                let register = self.var_register(variable);
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
