use std::{collections::{HashMap, BTreeSet, HashSet}, mem, fmt};
use crate::{frontend::buildtree::{BuildTree, BTTopDefn}, model::{linear::{LinearStatement, LinearStatementValue}, checkstypes::{AtomicTypeSpec, TypeSpec, TypeRestriction}, codeblocks::CodeArgument}, controller::source::ParsePosition, util::equiv::EquivalenceClass};

#[derive(PartialEq,Eq,Clone,PartialOrd,Ord)]
pub(crate) enum NarrowType {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
}

impl NarrowType {
    pub(crate) fn meets_restriction(&self, restr: &TypeRestriction) -> bool {
        match (self,restr) {
            (NarrowType::Atomic(a), TypeRestriction::Atomic(b)) => a == b,
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

fn add_sequences(output: &mut HashSet<TypeRestriction>, input: &HashSet<TypeRestriction>) {
    output.extend(input.iter().filter(|r| {
        match r {
            TypeRestriction::Sequence(_) => true,
            _ => false
        }
    }).cloned())
}

fn intersect_restrictions(a: &HashSet<TypeRestriction>, b: &HashSet<TypeRestriction>) -> HashSet<TypeRestriction> {
    let mut out = HashSet::new();
    let a_any = a.contains(&TypeRestriction::AnySequence);
    let b_any = b.contains(&TypeRestriction::AnySequence);
    if a_any && b_any { out.insert(TypeRestriction::AnySequence); }
    if a_any { add_sequences(&mut out,b); }
    if b_any { add_sequences(&mut out,a); }
    out.extend(a.intersection(b).cloned());
    out
}

#[derive(Clone,Debug)]
struct AtomPoss {
    sequence: bool,
    atom: bool,
    number: bool,
    string: bool,
    boolean: bool,
    any_handle: bool,
    specific_handles: BTreeSet<String>,
    restrictions: Option<HashSet<TypeRestriction>>
}

impl AtomPoss {
    fn apply_restrs(&self, narrow: &NarrowType) -> bool {
        if let Some(restrs) = &self.restrictions {
            for restr in restrs {
                if narrow.meets_restriction(restr) { return true; }
            }
            false
        } else {
            true
        }
    }

    fn atom_type_options(&self) -> Vec<AtomicTypeSpec> {
        let mut out = vec![];
        if self.boolean { out.push(AtomicTypeSpec::Boolean); }
        if self.number { out.push(AtomicTypeSpec::Number); }
        if self.string { out.push(AtomicTypeSpec::String); }
        if self.any_handle { out.push(AtomicTypeSpec::Handle("".to_string())); }
        out.extend(self.specific_handles.iter().map(|name| AtomicTypeSpec::Handle(name.to_string())));
        out
    }

    fn type_options(&self) -> Vec<NarrowType> {
        let mut out = vec![];
        let atoms = self.atom_type_options();
        if self.atom { out.extend(atoms.iter().map(|a| NarrowType::Atomic(a.clone()))) }
        if self.sequence { out.extend(atoms.iter().map(|a| NarrowType::Sequence(a.clone()))) }
        out
    }

    fn calc_type(&self) -> Result<NarrowType,String> {
        for narrow in self.type_options() {
            if self.apply_restrs(&narrow) { return Ok(narrow); }
        }
        return Err(format!("cannot deduce type/A"));
    }

    fn any() -> AtomPoss {
        AtomPoss { 
            sequence: true, atom: true,
            number: true, string: true, boolean: true, any_handle: true, 
            specific_handles: BTreeSet::new(),
            restrictions: None
        }
    }

    fn none() -> AtomPoss {
        AtomPoss { 
            sequence: false, atom: false,
            number: false, string: false, boolean: false, any_handle: false, 
            specific_handles: BTreeSet::new(),
            restrictions: None
        }
    }

    fn check_valid_bool(&self) -> bool {
        if !self.sequence && !self.atom { return false; }
        if !self.number && !self.string && !self.boolean && !self.any_handle && self.specific_handles.len() == 0 {
            return false;
        }
        if let Some(restrs) = &self.restrictions {
            if restrs.len() == 0 { return false; }
        }
        true
    }

    fn check_valid(&self) -> Result<(),String> {
        if self.check_valid_bool() { Ok(()) } else { Err(format!("cannot deduce type/B")) }
    }

    fn atom_to_seq(&self) -> Result<AtomPoss,String> {
        let mut out = self.clone();
        if !self.atom { return Err(format!("cannot deduce type")); }
        out.atom = false;
        out.sequence = true;
        if let Some(restr) = &self.restrictions {
            out.restrictions = Some(restr.iter().filter_map(|r| {
                match r {
                    TypeRestriction::Atomic(a) => Some(TypeRestriction::Sequence(a.clone())),
                    TypeRestriction::Sequence(_) => None,
                    TypeRestriction::AnySequence => None
                }
            }).collect());
        }
        out.check_valid()?;
        Ok(out)
    }

    fn seq_to_atom(&self) -> Result<AtomPoss,String> {
        let mut out = self.clone();
        if !self.sequence { return Err(format!("cannot deduce type/C")); }
        out.sequence = false;
        out.atom = true;
        if let Some(in_restr) = &self.restrictions {
            let mut restrs = HashSet::new();
            let mut any = false;
            for restr in in_restr {
                match restr {
                    TypeRestriction::Atomic(_) => {},
                    TypeRestriction::Sequence(s) => {
                        restrs.insert(TypeRestriction::Atomic(s.clone()));
                    },
                    TypeRestriction::AnySequence => {
                        any = true;
                    }
                }
            }
            out.restrictions = if any { None } else { Some(restrs) };
        }
        out.check_valid()?;
        Ok(out)
    }

    fn acceptable_atom(&self, a: &AtomicTypeSpec) -> bool {
        match a {
            AtomicTypeSpec::Number => self.number,
            AtomicTypeSpec::String => self.string,
            AtomicTypeSpec::Boolean => self.boolean,
            AtomicTypeSpec::Handle(h) => {
                self.specific_handles.contains(h) || self.any_handle
            }
        }
    }

    fn filter_acceptable(&mut self) {
        self.restrictions = self.restrictions.as_ref().map(|restrs| {
            restrs.iter().filter(|r| {
                match r {
                    TypeRestriction::Atomic(a) => {
                        self.atom && self.acceptable_atom(a)
                    },
                    TypeRestriction::Sequence(a) => {
                        self.sequence && self.acceptable_atom(a)
                    },
                    TypeRestriction::AnySequence => self.sequence
                }
            }).cloned().collect::<HashSet<_>>()
        });
    }

    fn sequence(&mut self) -> Result<(),String> {
        self.atom = false;
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }

    fn unify(&mut self, other: &AtomPoss) -> Result<(),String> {
        self.sequence &= other.sequence;
        self.atom &= other.atom;
        self.number &= other.number;
        self.string &= other.string;
        self.boolean &= other.boolean;
        match (self.any_handle,other.any_handle) {
            (true, false) => {
                self.specific_handles = other.specific_handles.clone();
            },
            (false, false) => {
                self.specific_handles = self.specific_handles.intersection(&other.specific_handles).cloned().collect();
            },
            _ => {}
        }
        self.any_handle &= other.any_handle;
        self.restrictions = match (&self.restrictions,&other.restrictions) {
            (None, None) => { None },
            (None, Some(r)) => { Some(r.clone()) },
            (Some(r), None) => { Some(r.clone()) },
            (Some(a), Some(b)) => {
                Some(intersect_restrictions(a,b))
            },
        };
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }

    fn restrict_by_type(&mut self, restrs: &[TypeRestriction]) -> Result<(),String> {
        let restrs = restrs.iter().cloned().collect::<HashSet<_>>();
        if let Some(self_restrs) = &mut self.restrictions {
            *self_restrs = intersect_restrictions(&restrs,&self_restrs);
        } else {
            self.restrictions = Some(restrs);
        }
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }

    fn restrict_by_spec(&mut self, seq: bool, spec: &AtomicTypeSpec) -> Result<(),String> {
        let mut new = Self::none();
        let ok = match spec {
            AtomicTypeSpec::Number => { new.number = true; self.number },
            AtomicTypeSpec::String => { new.string = true; self.string },
            AtomicTypeSpec::Boolean => { new.boolean = true; self.boolean }
            AtomicTypeSpec::Handle(h) => {
                new.specific_handles.insert(h.to_string());
                self.any_handle || self.specific_handles.contains(h)
            },
        };
        if !ok { return Err(format!("cannot deduce type/D")); }
        let ok = if seq { new.sequence = true; self.sequence } else { new.atom = true; self.atom };
        if !ok { return Err(format!("cannot deduce type/E")); }
        *self = new;
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }
}

struct NarrowTyping<'a> {
    bt: &'a BuildTree,
    block_index: &'a HashMap<usize,usize>,
    position: ParsePosition,
    possible: HashMap<usize,AtomPoss>,
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

    fn poss_for_reg(&mut self, reg: usize) -> &mut AtomPoss {
        self.seen.insert(reg);
        let reg = self.equivs.canon(reg);
        self.possible.entry(reg).or_insert_with(|| AtomPoss::any())
    }

    fn arg(&mut self, ties: &mut HashMap<String,Vec<(bool,usize)>>, spec: &CodeArgument, reg: usize) -> Result<(),String> {
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
            TypeSpec::SequenceWildcard(w) => {
                self.poss_for_reg(reg).sequence()?;
                ties.entry(w.to_string()).or_insert(vec![]).push((true,reg));
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
            self.arg(&mut ties,spec,*reg)?;
        }
        /* results */
        for (spec,reg) in block.results.iter().zip(rets.iter()) {
            self.arg(&mut ties,spec,*reg)?;
        }
        /* manage ties */
        for (_,tied) in ties.drain() {
            let mut wc = AtomPoss::any();
            let mut swc = AtomPoss::any();
            let mut seen_sw = false;
            for (sw,reg) in &tied {
                if *sw { seen_sw = true; }
                let poss = if *sw { &mut swc } else { &mut wc };
                poss.unify(&self.poss_for_reg(*reg))?;
            }
            if seen_sw {
                swc.unify(&wc.atom_to_seq()?)?;
                wc.unify(&swc.seq_to_atom()?)?;
            }
            for (sw,reg) in &tied {
                let poss = if *sw { &mut swc } else { &mut wc };
                self.poss_for_reg(*reg).unify(&poss)?;
            }
        }
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
            LinearStatementValue::WildEquiv(regs) => {
                let mut poss = AtomPoss::any();
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
            }
            _ => {}
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
