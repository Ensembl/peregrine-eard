use std::{collections::{HashMap, BTreeMap}, fmt};
use crate::{frontend::buildtree::{BTTopDefn, BuildTree}, model::{constants::FullConstant, operation::{OperationValue, Operation}, codeblocks::{CodeBlock, CodeModifier}}, test::testutil::sepfmt};
use super::{broadtyping::BroadType, narrowtyping::NarrowType};

#[derive(PartialEq,Eq,Clone,PartialOrd,Ord)]
pub(crate) enum KnownValue {
    Constant(FullConstant),
    Derived(usize,Vec<Box<KnownValue>>,usize)
}

impl fmt::Debug for KnownValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(c) => write!(f,"{:?}",c),
            Self::Derived(name, arg, pos) => 
                write!(f,"{}/{}({})",
                    *name,*pos,sepfmt(&mut arg.iter(),", ","")
                ),
        }
    }
}

struct KnownValues<'a> {
    bt: &'a BuildTree,
    broad: &'a HashMap<usize,BroadType>, 
    narrow: &'a HashMap<usize,NarrowType>,
    block_indexes: &'a HashMap<usize,usize>,
    reg_value: HashMap<(usize,BroadType,NarrowType),KnownValue>,
    value_reg: BTreeMap<(KnownValue,BroadType,NarrowType),usize>,
    alias_to: HashMap<usize,usize>,
    out: Vec<Operation>
}

impl<'a> KnownValues<'a> {
    fn new(bt: &'a BuildTree, broad: &'a HashMap<usize,BroadType>, narrow: &'a HashMap<usize,NarrowType>, block_indexes: &'a HashMap<usize,usize>) -> KnownValues<'a> {
        KnownValues {
            bt, block_indexes, broad, narrow,
            out: vec![],
            reg_value: HashMap::new(),
            value_reg: BTreeMap::new(),
            alias_to: HashMap::new()
        }
    }

    fn get_block(&mut self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_indexes.get(&call).unwrap_or(&0);
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn add_value(&mut self, reg: usize, value: &KnownValue) {
        let broad = self.broad.get(&reg).expect("missing broad type").clone();
        let narrow = self.narrow.get(&reg).expect("missing broad type").clone();
        self.value_reg.insert((value.clone(),broad.clone(),narrow.clone()),reg);
        self.reg_value.insert((reg,broad,narrow),value.clone());
    }

    fn alias_of(&self, reg: usize) -> usize {
        self.alias_to.get(&reg).cloned().unwrap_or(reg)
    }

    fn add_oper(&mut self, oper: &Operation) {
        match &oper.value {
            OperationValue::Constant(_,_) | OperationValue::Entry(_) => {
                self.out.push(oper.clone());
            },
            OperationValue::Code(call,name,rets,args) => {
                let args = args.iter().map(|reg| {
                    self.alias_of(*reg)
                }).collect::<Vec<_>>();
                self.out.push(Operation { 
                    position: oper.position.clone(), 
                    value: OperationValue::Code(*call,*name,rets.clone(),args)
                });
            }        
        }
    }

    fn arg_values(&mut self, args: &[usize]) -> Option<Vec<Box<KnownValue>>> {
        args.iter().map(|reg| {
            let reg = self.alias_of(*reg);
            let broad = self.broad.get(&reg).expect("missing broad type").clone();
            let narrow = self.narrow.get(&reg).expect("missing broad type").clone();    
            self.reg_value.get(&(reg,broad,narrow)).map(|x| Box::new(x.clone()))
        }).collect()
    }

    fn ret_values(&mut self, name: usize, const_args: &[Box<KnownValue>], rets: &[usize]) -> Option<Vec<usize>> {
        rets.iter().enumerate().map(|(i,reg)| {
            let broad = self.broad.get(&reg).expect("missing broad type").clone();
            let narrow = self.narrow.get(&reg).expect("missing broad type").clone();    
            let kv = KnownValue::Derived(name,const_args.to_vec(),i);
            self.value_reg.get(&(kv,broad,narrow)).cloned()
        }).collect()
    }

    fn add(&mut self, oper: &Operation) -> Result<(),String> {
        match &oper.value {
            OperationValue::Constant(reg,c) => {
                let broad = self.broad.get(&reg).expect("missing broad type").clone();
                let narrow = self.narrow.get(&reg).expect("missing broad type").clone();    
                let kv = KnownValue::Constant(c.clone());
                if let Some(stored) = self.value_reg.get(&(kv.clone(),broad,narrow)) {
                    self.alias_to.insert(*reg,*stored);
                } else {
                    self.add_value(*reg,&kv);
                    self.add_oper(oper);
                }
            },
            OperationValue::Entry(_) => {
                self.add_oper(oper);
            }
            OperationValue::Code(call,name,rets,args) => {
                let block = self.get_block(*call,*name);
                if !block.modifiers.contains(&CodeModifier::World) {
                    if let Some(const_args) = self.arg_values(args) {
                        if let Some(existing_rets) = self.ret_values(*name,&const_args,rets) {
                            /* All args and rets values are known: No run */
                            for (new,old) in rets.iter().zip(existing_rets.iter()) {
                                self.alias_to.insert(*new,*old);
                            }
                        } else {
                            /* All args are known but not all rest: run and record */
                            for (i,reg) in rets.iter().enumerate() {
                                let kv = KnownValue::Derived(*name,const_args.clone(),i);
                                self.add_value(*reg,&kv);
                            }
                            self.add_oper(oper);
                        }
                        return Ok(());
                    }
                }
                /* Not all args known: run and no record */
                self.add_oper(oper);
            }
        }
        Ok(())
    }
    
    fn take(self) -> Vec<Operation> { self.out }

    #[cfg(test)]
    fn test_take(self) -> (Vec<Operation>,HashMap<(usize,BroadType,NarrowType),KnownValue>) { (self.out,self.reg_value) }
}

#[cfg(test)]
pub(crate) fn test_reuse(bt: &BuildTree, broad: &HashMap<usize,BroadType>, narrow: &HashMap<usize,NarrowType>, block_indexes: &HashMap<usize,usize>, opers: &[Operation]) -> Result<(Vec<Operation>,HashMap<(usize,BroadType,NarrowType),KnownValue>),String> {
    let mut known = KnownValues::new(bt,broad,narrow,block_indexes);
    for oper in opers {
        known.add(oper)?;
    }
    Ok(known.test_take())
}

pub(crate) fn reuse(bt: &BuildTree, broad: &HashMap<usize,BroadType>, narrow: &HashMap<usize,NarrowType>, block_indexes: &HashMap<usize,usize>, opers: &[Operation], verbose: bool) -> Result<Vec<Operation>,String> {
    let mut known = KnownValues::new(bt,broad,narrow,block_indexes);
    for oper in opers {
        known.add(oper)?;
    }
    if verbose {
        eprintln!("memoizing side-effect free expressions left {} statements",known.out.len());
    }
    Ok(known.take())
}
