/* Generation converts the single-assignment form of operations into a sequence of opcodes
 * which can reuse registers and use modified forms.
 * 
 * First we record the last use of a register. 
 */

use std::{sync::Arc, collections::{HashMap, BTreeSet, BTreeMap, HashSet}};
use crate::{model::{Step, Operation, OperationValue, FullConstant, CodeImplArgument, CodeReturn, Opcode}, frontend::{parsetree::at, buildtree::{BTTopDefn, BuildTree}}, codeblocks::{CodeBlock, ImplBlock}, middleend::narrowtyping::NarrowType};

struct NewRegister(usize);

struct RegAllocate {
    const_scrapheap: BTreeMap<FullConstant,usize>,
    other_scrapheap: BTreeSet<usize>,
    next_reg: usize
}

impl RegAllocate {
    fn new() -> RegAllocate {
        RegAllocate {
            const_scrapheap: BTreeMap::new(),
            other_scrapheap: BTreeSet::new(),
            next_reg: 0
        }
    }

    fn scavange(&mut self, value: &FullConstant) -> Option<usize> {
        self.const_scrapheap.remove(value)
    }

    fn allocate(&mut self) -> usize {
        if let Some(reg) = self.other_scrapheap.iter().next().cloned() {
            self.other_scrapheap.remove(&reg);
            return reg;
        }
        if let Some((c,reg)) = self.const_scrapheap.iter().next().map(|(c,r)| (c.clone(),r.clone())) {
            self.const_scrapheap.remove(&c);
            return reg;
        }
        self.next_reg += 1;
        self.next_reg
    }

    fn free(&mut self, reg: usize, value: Option<FullConstant>) {
        if let Some(c) = value {
            self.const_scrapheap.insert(c,reg);
        } else {
            self.other_scrapheap.insert(reg);
        }
    }
}

struct RegMap {
    old_to_new: HashMap<usize,usize>,
    const_to_new: BTreeMap<FullConstant,HashSet<usize>>,
    new_to_const: HashMap<usize,FullConstant>
}

impl RegMap {
    fn new() -> RegMap {
        RegMap {
            old_to_new: HashMap::new(),
            const_to_new: BTreeMap::new(),
            new_to_const: HashMap::new()
        }
    }

    fn old_to_new(&self, reg: usize) -> Option<usize> {
        self.old_to_new.get(&reg).cloned()
    }

    fn const_to_new(&mut self, c: &FullConstant) -> &mut HashSet<usize> {
        self.const_to_new.entry(c.clone()).or_insert_with(|| HashSet::new())
    }

    fn new_with_const(&self, c: &FullConstant) -> Option<usize> {
        self.const_to_new.get(c).and_then(|e| e.iter().next().cloned())
    }

    fn set(&mut self, old: usize, new: usize, c: Option<&FullConstant>) {
        if let Some(old_value) = self.new_to_const.remove(&new) {
            self.const_to_new(&old_value).remove(&new);
        }
        self.old_to_new.insert(old,new);
        if let Some(c) = c {
            self.const_to_new(c).insert(new);
            self.new_to_const.insert(new,c.clone());
        }
    }
}

struct Generate<'a> {
    /* usual suspects */
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    narrow: &'a HashMap<usize,NarrowType>,
    position: Option<(Arc<Vec<String>>,usize)>,
    out: Vec<Step>,
    /**/
    constants: HashMap<usize,FullConstant>, // from oldreg
    unborn_constants: HashSet<usize>,
    last_use: HashMap<usize,usize>,
    regmap: RegMap,
    reg_alloc: RegAllocate
}

impl<'a> Generate<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, narrow: &'a HashMap<usize,NarrowType>) -> Generate<'a> {
        Generate {
            bt, block_index, narrow,
            position: None,
            constants: HashMap::new(),
            unborn_constants: HashSet::new(),
            last_use: HashMap::new(),
            regmap: RegMap::new(),
            reg_alloc: RegAllocate::new(),
            out: vec![]
        }
    }

    fn error_at(&self, msg: &str) -> String {
        self.position.as_ref().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
    }

    fn find_last(&mut self, index: usize, oper: &Operation) {
        self.position = Some(oper.position.clone());
        match &oper.value {
            OperationValue::Constant(_, _) => {},
            OperationValue::Code(_, _, _,args) => {
                for arg in args {
                    self.last_use.insert(*arg,index);
                }
            },
        }
    }

    fn get_block(&self, call: usize, name: usize) -> &CodeBlock {
        let block_index = *self.block_index.get(&call).expect("missing block index");
        let code_block = match self.bt.get_by_index(name).expect("missing code block") {
            BTTopDefn::FuncProc(_) => { panic!("code index to non-code"); },
            BTTopDefn::Code(c) => c
        };
        code_block.get_block(block_index)
    }

    fn get_impl(&mut self, block: &CodeBlock, index: usize, args: &[usize]) -> Result<ImplBlock,String> {
        let modify_pattern = args.iter().map(|reg_old| {
            self.last_use.get(reg_old).and_then(|modifiable| {
                if *modifiable == index { Some(*reg_old) } else { None }
            })
        }).collect::<Vec<_>>();
        let constants = args.iter().map(|reg_old| {
            self.constants.get(reg_old).cloned()
        }).collect::<Vec<_>>();
        let narrow = args.iter().map(|reg_old|
            self.narrow.get(reg_old).expect("missing type").clone()
        ).collect::<Vec<_>>();
        let imps = block.choose_imps(&constants,Some(&narrow),Some(&modify_pattern));
        if imps.len() == 0 {
            return Err("no opcode implementation matching type".to_string());
        }
        Ok(imps[0].clone())
    }

    fn free_reg(&mut self, reg: usize) {
        let c = self.constants.get(&reg).cloned();
        if let Some(new_reg) = self.regmap.old_to_new(reg) {
            self.reg_alloc.free(new_reg,c);
        }
    }

    fn make_opcode(&mut self, imp: &ImplBlock, index: usize, opcode: &Opcode, rets: &[usize], args: &[usize]) -> Result<Step,String> {
        let def_to_reg = self.map_regs(imp,index,rets,args)?;
        let opargs = opcode.1.iter().map(|d| 
            *def_to_reg.get(d).expect("missing register")
        ).collect::<Vec<_>>();
        Ok(Step::Opcode(opcode.0,opargs))
    }

    /* Map of name of argument in decl to new register name. Also tidies used regs */
    fn map_regs(&mut self, imp: &ImplBlock, index: usize, rets: &[usize], args: &[usize]) -> Result<HashMap<usize,usize>,String> {
        let mut def_to_reg = HashMap::new();
        let mut reused_def = HashSet::new();
        for (arg,reg) in imp.arguments.iter().zip(args.iter()) {
            match arg {
                CodeImplArgument::Register(d) => {
                    let existing_new = self.constants.get(reg)
                        .and_then(|c| self.regmap.new_with_const(c));
                    let new_reg = if let Some(r) = existing_new {
                        r
                    } else {
                        self.birth_constants(*reg);                    
                        self.regmap.old_to_new(*reg).unwrap()
                    };
                    def_to_reg.insert(d.reg_id,new_reg);
                },
                CodeImplArgument::Constant(_) => {}
            }
        }
        for (ret,reg) in imp.results.iter().zip(rets.iter()) {
            match ret {
                CodeReturn::Register(d) => {
                    let new_ret = self.reg_alloc.allocate();
                    def_to_reg.insert(d.reg_id,new_ret);
                    self.regmap.set(*reg,new_ret,None);
                },
                CodeReturn::Repeat(d) => {
                    let new_reg = def_to_reg.get(d).expect("missing repetition register");
                    self.regmap.set(*reg,*new_reg,None);
                    reused_def.insert(*d);
                }
            }
        }
        for (arg,reg) in imp.arguments.iter().zip(args.iter()) {
            match arg {
                CodeImplArgument::Register(d) => {
                    if let Some(line) = self.last_use.get(reg) {
                        if *line == index && !reused_def.contains(&d.reg_id) {
                            self.free_reg(*reg);
                        }
                    }
                },
                CodeImplArgument::Constant(_) => {}
            }
        }
        for reg in rets.iter() {
            if self.last_use.get(reg).is_none() {
                if *reg > 0 {
                    self.free_reg(*reg);
                }
            }
        }
        Ok(def_to_reg)
    }

    fn code(&mut self, block: &CodeBlock, index: usize, rets: &[usize], args: &[usize]) -> Result<Option<Step>,String> {
        let imp = self.get_impl(block,index,args)?;
        let def_to_reg = self.map_regs(&imp,index,rets,args)?;
        Ok(imp.command.as_ref().map(|opcode| {
            let opargs = opcode.1.iter().map(|d| 
                *def_to_reg.get(d).expect("missing register")
            ).collect::<Vec<_>>();
            Step::Opcode(opcode.0,opargs)
        }))
    }

    fn birth_constants(&mut self, reg: usize) -> usize {
        if self.unborn_constants.remove(&reg) {
            let c = self.constants.get(&reg).expect("missing constant");
            let new_reg = if let Some(r) = self.reg_alloc.scavange(c) {
                r
            } else {
                self.reg_alloc.allocate()
            };
            self.regmap.set(reg,new_reg,Some(c));
            self.out.push(Step::Constant(new_reg,c.clone()));
        }
        reg
    }

    fn add(&mut self, index: usize, oper: &Operation) -> Result<(),String> {
        self.position = Some(oper.position.clone());
        match &oper.value {
            OperationValue::Constant(reg, c) => {
                self.constants.insert(*reg,c.clone());
                self.unborn_constants.insert(*reg);
            },
            OperationValue::Code(call,name,rets,args) =>  {
                let block = self.get_block(*call,*name).clone();
                if let Some(step) = self.code(&block,index,rets,args)? {
                    self.out.push(step);
                }
            }
        }
        Ok(())
    }

    fn take(self) -> Vec<Step> { self.out }
}

pub(crate) fn generate(bt: &BuildTree, block_index: &HashMap<usize,usize>, narrow: &HashMap<usize,NarrowType>, opers: &[Operation]) -> Result<Vec<Step>,String> {
    let mut generate = Generate::new(bt,block_index,narrow);
    for (i,oper) in opers.iter().enumerate() {
        generate.find_last(i,oper);
    }
    for (i,oper) in opers.iter().enumerate() {
        generate.add(i,&oper).map_err(|e| generate.error_at(&e))?;
    }
    Ok(generate.take())
}
