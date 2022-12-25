use std::{collections::{HashMap, BTreeSet, HashSet}, mem, fmt};
use crate::{frontend::buildtree::{BuildTree, BTTopDefn}, model::{linear::{LinearStatement, LinearStatementValue}, checkstypes::{AtomicTypeSpec, TypeSpec, TypeRestriction, intersect_restrictions}, codeblocks::CodeArgument}, controller::source::ParsePosition, util::equiv::EquivalenceClass};

use super::possible::NarrowPoss;

#[derive(PartialEq,Eq,Clone,PartialOrd,Ord)]
pub(crate) enum NarrowType {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
}

impl NarrowType {
    pub(crate) fn meets_restriction(&self, restr: &TypeRestriction) -> bool {
        match (self,restr) {
            (NarrowType::Atomic(a), TypeRestriction::Atomic(b)) => a == b,
            (NarrowType::Atomic(_),TypeRestriction::AnyAtomic) => true,
            (NarrowType::Sequence(a), TypeRestriction::Sequence(b)) => a == b,
            (NarrowType::Sequence(_), TypeRestriction::AnySequence) => true,
            _ => false
        }
    }
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
    position: ParsePosition,
    possible: HashMap<usize,NarrowPoss>,
    equivs: EquivalenceClass<usize>,
    seen: HashSet<usize>
}

impl<'a> NarrowTyping<'a> {
    fn new(bt: &'a BuildTree, block_index: &'a HashMap<usize,usize>) -> NarrowTyping<'a> {
        NarrowTyping {
            bt, block_index,
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

    fn spec(&mut self, ties: &mut HashMap<String,Vec<(bool,usize)>>, spec: &CodeArgument, reg: usize) -> Result<(),String> {
        match &spec.arg_type {
            TypeSpec::Atomic(a) => {
                self.poss_for_reg(reg).restrict_by_spec(false,a)?;
            },
            TypeSpec::Sequence(a) => {
                self.poss_for_reg(reg).restrict_by_spec(true,a)?;
            },
            TypeSpec::Wildcard(w) => {
                ties.entry(w.to_string()).or_insert(vec![]).push((false,reg));
            },
            TypeSpec::AtomWildcard(w) => {
                self.poss_for_reg(reg).atomic()?;
                ties.entry(w.to_string()).or_insert(vec![]).push((false,reg));
            },
            TypeSpec::SequenceWildcard(w) => {
                self.poss_for_reg(reg).sequence()?;
                ties.entry(w.to_string()).or_insert(vec![]).push((true,reg));
            }
        }
        Ok(())
    }

    fn unify(&mut self, tied: &[(bool,usize)]) -> Result<(),String> {
        let mut wc = NarrowPoss::any();
        let mut swc = NarrowPoss::any();
        let mut seen_sw = false;
        for (sw,reg) in tied {
            if *sw { seen_sw = true; }
            let poss = if *sw { &mut swc } else { &mut wc };
            poss.unify(&self.poss_for_reg(*reg))?;
        }
        if seen_sw {
            swc.unify(&wc.atom_to_seq()?)?;
            wc.unify(&swc.seq_to_atom()?)?;
        }
        for (sw,reg) in tied {
            let poss = if *sw { &mut swc } else { &mut wc };
            self.poss_for_reg(*reg).unify(&poss)?;
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
            self.spec(&mut ties,spec,*reg)?;
        }
        /* results */
        for (spec,reg) in block.results.iter().zip(rets.iter()) {
            self.spec(&mut ties,spec,*reg)?;
        }
        /* manage ties */
        for (_,tied) in ties.drain() {
            self.unify(&tied)?;
        }
        Ok(())
    }

    fn signature(&mut self, sig: &[(usize,Vec<TypeSpec>)]) -> Result<(),String> {
        for (reg,specs) in sig {

        }
        //todo!();
        Ok(())
    }

    fn add(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = stmt.position.clone();
        match &stmt.value {
            LinearStatementValue::Code(call,name,rets,args) => { 
                self.code(*call,*name,rets,args)?;
            },
            LinearStatementValue::Type(reg,restrs) => {
                self.poss_for_reg(*reg).restrict_by_type(restrs)?;
            },
            LinearStatementValue::SameType(regs) => {
                let mut poss = NarrowPoss::any();
                for reg in regs {
                    poss.unify(&self.poss_for_reg(*reg))?;
                }
                for reg in regs {
                    self.poss_for_reg(*reg).unify(&poss)?;
                }
                let mut rest = regs.to_vec();
                if let Some(first) = rest.pop() {
                    for another in &rest {
                        self.equivs.equiv(first,*another);
                    }
                }
            },
            LinearStatementValue::Constant(reg,c) => {
                let spec = c.to_atomic_type();
                self.poss_for_reg(*reg).restrict_by_spec(false,&spec)?;
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
            let narrow = self.poss_for_reg(reg).calc_type()?;
            out.insert(reg,narrow);
        }
        Ok(out)
    }
}

pub(crate) fn narrow_type(bt: &BuildTree, block_index: &HashMap<usize,usize>, stmts: &[LinearStatement]) -> Result<HashMap<usize,NarrowType>,String> {
    let mut typing = NarrowTyping::new(bt,block_index);
    for stmt in stmts {
        typing.add(stmt).map_err(|e| typing.position.message(&e))?;
    }
    typing.finalise()
}
