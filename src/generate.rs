/* Generation converts the single-assignment form of operations into a sequence of opcodes
 * which can reuse registers and use modified forms.
 * 
 * First we record the last use of a register. 
 */

use std::{collections::{HashMap, BTreeSet, BTreeMap, HashSet}};
use crate::{model::{Step, Operation, OperationValue, FullConstant, CodeImplArgument, CodeReturn}, frontend::{buildtree::{BTTopDefn, BuildTree}}, codeblocks::{CodeBlock, ImplBlock}, middleend::narrowtyping::NarrowType, source::ParsePosition};

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
struct NewRegister(usize);

struct RegAllocate {
    const_scrapheap: BTreeMap<FullConstant,NewRegister>,
    other_scrapheap: BTreeSet<NewRegister>,
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

    fn scavange(&mut self, value: &FullConstant) -> Option<NewRegister> {
        self.const_scrapheap.remove(value)
    }

    fn allocate(&mut self) -> NewRegister {
        if let Some(reg) = self.other_scrapheap.iter().next().cloned() {
            self.other_scrapheap.remove(&reg);
            return reg;
        }
        if let Some((c,reg)) = self.const_scrapheap.iter().next().map(|(c,r)| (c.clone(),r.clone())) {
            self.const_scrapheap.remove(&c);
            return reg;
        }
        self.next_reg += 1;
        NewRegister(self.next_reg)
    }

    fn free(&mut self, reg: NewRegister, value: Option<FullConstant>) {
        if let Some(c) = value {
            self.const_scrapheap.insert(c,reg);
        } else {
            self.other_scrapheap.insert(reg);
        }
    }
}

struct CodeMapping<'a> {
    def_to_reg: HashMap<usize,NewRegister>, // name in definition to new register name
    reused_def: HashSet<usize>, // given name in definition is repeated in result
    imp: &'a ImplBlock,
    index: usize,
    rets: &'a [usize],
    args: &'a [usize]
}

struct Generate<'a> {
    /* usual suspects */
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    narrow: &'a HashMap<usize,NarrowType>,
    position: ParsePosition,
    out: Vec<Step>,
    /**/
    constants: HashMap<usize,FullConstant>, // from oldreg
    unborn_constants: HashSet<usize>,
    last_use: HashMap<usize,usize>,
    regmap: HashMap<usize,NewRegister>,
    reg_alloc: RegAllocate
}

impl<'a> Generate<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, narrow: &'a HashMap<usize,NarrowType>) -> Generate<'a> {
        Generate {
            bt, block_index, narrow,
            position: ParsePosition::empty("called"),
            constants: HashMap::new(),
            unborn_constants: HashSet::new(),
            last_use: HashMap::new(),
            regmap: HashMap::new(),
            reg_alloc: RegAllocate::new(),
            out: vec![]
        }
    }

    fn find_last(&mut self, index: usize, oper: &Operation) {
        self.position = oper.position.clone();
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
        if let Some(new_reg) = self.regmap.get(&reg) {
            self.reg_alloc.free(*new_reg,c);
        }
    }

    fn birth_and_map_args(&mut self, mapping: &mut CodeMapping) {
        for (arg,reg) in mapping.imp.arguments.iter().zip(mapping.args.iter()) {
            match arg {
                CodeImplArgument::Register(d) => {
                    self.birth_constants(*reg);                    
                    let new_reg = self.regmap.get(reg).expect("missing constant");
                    mapping.def_to_reg.insert(d.reg_id,*new_reg);
                },
                CodeImplArgument::Constant(_) => {}
            }
        }
    }

    fn map_rets_and_detect_repeats(&mut self, mapping: &mut CodeMapping) {
        for (ret,reg) in mapping.imp.results.iter().zip(mapping.rets.iter()) {
            match ret {
                CodeReturn::Register(d) => {
                    let new_ret = self.reg_alloc.allocate();
                    mapping.def_to_reg.insert(d.reg_id,new_ret);
                    self.regmap.insert(*reg,new_ret);
                },
                CodeReturn::Repeat(d) => {
                    let new_reg = mapping.def_to_reg.get(d).expect("missing repetition register");
                    self.regmap.insert(*reg,*new_reg);
                    mapping.reused_def.insert(*d);
                }
            }
        }
    }

    fn free_last_arg_use(&mut self, mapping: &mut CodeMapping) {
        for (arg,reg) in mapping.imp.arguments.iter().zip(mapping.args.iter()) {
            match arg {
                CodeImplArgument::Register(d) => {
                    if let Some(line) = self.last_use.get(reg) {
                        if *line == mapping.index && !mapping.reused_def.contains(&d.reg_id) {
                            self.free_reg(*reg);
                        }
                    }
                },
                CodeImplArgument::Constant(_) => {}
            }
        }
    }

    fn free_never_used_rets(&mut self, mapping: &mut CodeMapping) {
        for reg in mapping.rets.iter() {
            if self.last_use.get(reg).is_none() {
                if *reg > 0 {
                    self.free_reg(*reg);
                }
            }
        }
    }

    /* Map of name of argument in decl to new register name. Also tidies used regs */
    fn map_regs(&mut self, imp: &ImplBlock, index: usize, rets: &[usize], args: &[usize]) -> Result<HashMap<usize,NewRegister>,String> {
        let mut mapping = CodeMapping {
            imp, index, rets, args, 
            def_to_reg: HashMap::new(),
            reused_def: HashSet::new()
        };
        self.birth_and_map_args(&mut mapping);
        self.map_rets_and_detect_repeats(&mut mapping);
        self.free_last_arg_use(&mut mapping);
        self.free_never_used_rets(&mut mapping);
        Ok(mapping.def_to_reg)
    }

    fn code(&mut self, block: &CodeBlock, index: usize, rets: &[usize], args: &[usize]) -> Result<Option<Step>,String> {
        let imp = self.get_impl(block,index,args)?;
        let def_to_reg = self.map_regs(&imp,index,rets,args)?;
        Ok(imp.command.as_ref().map(|opcode| {
            let opargs = opcode.1.iter().map(|d| 
                def_to_reg.get(d).expect("missing register").0
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
            self.regmap.insert(reg,new_reg);
            self.out.push(Step::Constant(new_reg.0,c.clone()));
        }
        reg
    }

    fn add(&mut self, index: usize, oper: &Operation) -> Result<(),String> {
        self.position = oper.position.clone();
        match &oper.value {
            OperationValue::Constant(reg, c) => {
                if let Some(new_reg) = self.reg_alloc.scavange(c) {
                    self.regmap.insert(*reg,new_reg);
                } else {
                    self.unborn_constants.insert(*reg);    
                }
                self.constants.insert(*reg,c.clone());
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
        generate.add(i,&oper).map_err(|e| generate.position.message(&e))?;
    }
    Ok(generate.take())
}
