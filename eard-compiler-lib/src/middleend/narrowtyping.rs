use std::{collections::{HashMap, HashSet}, mem, fmt};
use crate::{frontend::buildtree::{BuildTree, BTTopDefn}, model::{linear::{LinearStatement, LinearStatementValue}, checkstypes::{AtomicTypeSpec, TypeSpec}, codeblocks::CodeArgument}, controller::source::ParsePosition, util::equiv::EquivalenceClass};
use super::{possible::NarrowPoss, broadtyping::BroadType};

#[derive(PartialEq,Eq,Clone,PartialOrd,Ord)]
pub(crate) enum NarrowType {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
}

impl fmt::Debug for NarrowType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic(a) => write!(f,"{:?}",a),
            Self::Sequence(s) => write!(f,"seq({:?})",s),
        }
    }
}

struct NarrowTyping<'a> {
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    broad: &'a HashMap<usize,BroadType>,
    position: ParsePosition,
    possible: HashMap<usize,NarrowPoss>,
    equivs: EquivalenceClass<usize>,
    seen: HashSet<usize>
}

impl<'a> NarrowTyping<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>, broad: &'a HashMap<usize,BroadType>) -> NarrowTyping<'a> {
        NarrowTyping {
            bt, block_index, broad,
            position: ParsePosition::empty("called"),
            possible: HashMap::new(),
            equivs: EquivalenceClass::new(),
            seen: HashSet::new(),
        }
    }

    fn poss_for_reg(&mut self, reg: usize) -> &mut NarrowPoss {
        self.seen.insert(reg);
        let reg = self.equivs.canon(reg);
        self.possible.entry(reg).or_insert_with(|| NarrowPoss::any())
    }

    fn spec(&mut self, ties: &mut HashMap<String,Vec<(bool,usize)>>, spec: &CodeArgument, reg: usize, broad: &BroadType) -> Result<(),String> {
        match &spec.arg_type {
            TypeSpec::Atomic(a) => {
                self.poss_for_reg(reg).restrict_by_spec(a)?;
            },
            TypeSpec::Sequence(a) => {
                self.poss_for_reg(reg).restrict_by_spec(a)?;
            },
            TypeSpec::Wildcard(w) => {
                ties.entry(w.to_string()).or_insert(vec![]).push((false,reg));
            },
            TypeSpec::AtomWildcard(w) => {
                match broad {
                    BroadType::Atomic => {},
                    BroadType::Sequence => { return Err(format!("cannot unify types/A")); },
                }
                ties.entry(w.to_string()).or_insert(vec![]).push((false,reg));
            },
            TypeSpec::SequenceWildcard(w) => {
                match broad {
                    BroadType::Sequence => {},
                    BroadType::Atomic => { return Err(format!("cannot unify types {}/B",w)); },
                }
                ties.entry(w.to_string()).or_insert(vec![]).push((true,reg));
            }
        }
        Ok(())
    }

    fn unify(&mut self, tied: &[(bool,usize)]) -> Result<(),String> {
        let mut wc = NarrowPoss::any();
        for (_,reg) in tied {
            wc.unify(&self.poss_for_reg(*reg))?;
        }
        for (_,reg) in tied {
            self.poss_for_reg(*reg).unify(&wc)?;
        }
        Ok(())
    }

    fn set_equivalent(&mut self, regs: &[usize]) -> Result<(),String> {
        /* create unified type of everything passed in */
        let mut poss = NarrowPoss::any();
        for reg in regs {
            poss.unify(&self.poss_for_reg(*reg))?;
        }
        /* unify all the records */
        for reg in regs {
            self.poss_for_reg(*reg).unify(&poss)?;
        }
        /* set all to equivalent for future use */
        let mut rest = regs.to_vec();
        if let Some(first) = rest.pop() {
            for another in &rest {
                self.equivs.equiv(first,*another);
            }
        }
        Ok(())
    }

    fn code(&mut self, call: usize, name: usize, rets: &[usize], args: &[usize]) -> Result<(),String> {
        let block_index = *self.block_index.get(&call).unwrap_or(&0);
        let block = match self.bt.get_by_index(name)? {
            BTTopDefn::Code(c) => c.get_block(block_index),
            _ => { panic!("didn't get code with code index"); }
        };
        /* arguments */
        let mut ties = HashMap::new();
        for (spec,reg) in block.arguments.iter().zip(args.iter()) {
            let broad = self.broad.get(reg).expect("missing broad type");
            self.spec(&mut ties,spec,*reg,broad)?;
        }
        /* results */
        for (spec,reg) in block.results.iter().zip(rets.iter()) {
            let broad = self.broad.get(reg).expect("missing broad type");
            self.spec(&mut ties,spec,*reg,broad)?;
        }
        /* manage ties */
        for (_,tied) in ties.drain() {
            self.unify(&tied)?;
        }
        Ok(())
    }

    fn signature(&mut self, sig: &[(usize,Vec<TypeSpec>)]) -> Result<(),String> {
        /* resolve atomic type of wilds: note, guaranteed at most one */
        let mut wilds = HashMap::new();
        for (reg,specs) in sig {
            for spec in specs {
                let wild = match spec {
                    TypeSpec::Wildcard(w) => Some(w),
                    TypeSpec::AtomWildcard(w) => Some(w),
                    TypeSpec::SequenceWildcard(w) => Some(w),
                    _ => None
                };
                if let Some(wild) = wild {
                    if let Some(old_reg) = wilds.get(wild) {
                        self.set_equivalent(&[*old_reg,*reg])?;
                    } else {
                        wilds.insert(wild.to_string(),*reg);
                    }
                }
            }
            self.poss_for_reg(*reg).unify(&NarrowPoss::from_type_specs(specs))?;
        }
        Ok(())
    }

    fn add(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Code(call,name,rets,args) => { 
                self.code(*call,*name,rets,args)?;
            },
            LinearStatementValue::Constant(reg,c) => {
                let spec = c.to_atomic_type();
                self.poss_for_reg(*reg).restrict_by_spec(&spec)?;
            },
            LinearStatementValue::Signature(s) => {
                self.signature(s)?;
            },
            LinearStatementValue::Check(_, _, _, _, _) => {},
            LinearStatementValue::Copy(_, _) => {},
            LinearStatementValue::Entry(_) => {},
        }
        Ok(())
    }

    fn finalise(&mut self) -> Result<HashMap<usize,NarrowType>,String> {
        let mut out = HashMap::new();
        let seen = mem::replace(&mut self.seen,HashSet::new());
        for reg in seen {
            let broad = self.broad.get(&reg).expect("missing broad type for register");
            let narrow = self.poss_for_reg(reg).calc_type(broad)?;
            out.insert(reg,narrow);
        }
        Ok(out)
    }
}

pub(crate) fn narrow_type(bt: &BuildTree, block_index: &HashMap<usize,usize>, broad: &HashMap<usize,BroadType>, stmts: &[LinearStatement]) -> Result<HashMap<usize,NarrowType>,String> {
    let mut typing = NarrowTyping::new(bt,block_index,broad);
    for stmt in stmts {
        typing.add(stmt).map_err(|e| typing.position.message(&e))?;
    }
    typing.finalise()
}
