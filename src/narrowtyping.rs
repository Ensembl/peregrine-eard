/* A narrow typing involves determiing which sequence to use for broad "sequence" types.
 * Again we proceed linearly through the program, leaving in our wake fully typed values.
 * Restrictions on which to use can only be imposed by code and type statements. Arguments to
 * a code statement can interact with each other to restrict thier wildcarded partners. They also
 * fully determine the type of their results.
 */

use std::{collections::{HashMap, HashSet}, sync::Arc, fmt};
use crate::{model::{AtomicTypeSpec, LinearStatement, LinearStatementValue, TypeSpec, TypeRestriction, CodeArgument}, frontend::{parsetree::at, buildtree::{BuildTree, BTTopDefn}}, broadtyping::BroadType, equiv::{EquivalenceClass, EquivalenceMap}, codeblocks::CodeBlock};

#[derive(Clone,Debug)]
enum WildcardType {
    Atomic(AtomicTypeSpec),
    Sequence(AtomicTypeSpec),
    AnySequence,
    AnyAtomic,
    Any
}

impl WildcardType {
    fn broad(&mut self, bt: &BroadType) -> Result<(),String> {
        let ok = match (self.clone(),bt) {
            (WildcardType::Atomic(a), BroadType::Atomic(b)) => &a == b,
            (WildcardType::Atomic(_), BroadType::Sequence) => false,
            (WildcardType::Sequence(_), BroadType::Atomic(_)) => false,
            (WildcardType::Sequence(_), BroadType::Sequence) => true,
            (WildcardType::AnySequence, BroadType::Atomic(_)) => false,
            (WildcardType::AnySequence, BroadType::Sequence) => true,
            (WildcardType::AnyAtomic, BroadType::Atomic(a)) => {
                *self = WildcardType::Atomic(a.clone());
                true
            },
            (WildcardType::AnyAtomic, BroadType::Sequence) => false,
            (WildcardType::Any, BroadType::Atomic(a)) => {
                *self = WildcardType::Atomic(a.clone());
                true
            },
            (WildcardType::Any, BroadType::Sequence) => {
                *self = WildcardType::AnySequence;
                true
            }
        };
        if ok { Ok(()) } else { Err(format!("type mismatch/C {:?} {:?}",self,bt)) }
    }

    fn atomic(&mut self) -> Result<(),String> {
        let ok = match self.clone() {
            WildcardType::Atomic(_) => true,
            WildcardType::Sequence(_) => false,
            WildcardType::AnySequence => false,
            WildcardType::AnyAtomic => true,
            WildcardType::Any => {
                *self = WildcardType::AnyAtomic;
                true
            }
        };
        if ok { Ok(()) } else { Err(format!("type mismatch/H {:?}",self)) }
    }

    fn restricted_atomic(&self) -> Result<Option<&AtomicTypeSpec>,String> {
        Ok(match self {
            WildcardType::Atomic(a) => Some(a),
            WildcardType::AnyAtomic => None,
            WildcardType::Any => None,
            _ => {
                return Err("type mismatch/B".to_string())
            }
        })
    }
}

#[derive(PartialEq,Eq,Clone)]
pub enum NarrowType {
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

impl NarrowType {
    fn atomic_to_sequence(&self) -> Result<NarrowType,String> {
        match self {
            NarrowType::Atomic(a) => Ok(NarrowType::Sequence(a.clone())),
            NarrowType::Sequence(_) => Err("type mismatch/A".to_string())
        }
    }
}

struct NarrowTyping<'a> {
    bt: &'a BuildTree,
    broad: &'a HashMap<usize,BroadType>,
    block_index: &'a HashMap<usize,usize>,
    position: Option<(Arc<Vec<String>>,usize)>,
    possible: EquivalenceMap<usize,Vec<AtomicTypeSpec>,String>
}

impl<'a> NarrowTyping<'a> {
    fn new(bt: &'a BuildTree, broad: &'a HashMap<usize,BroadType>, block_index: &'a HashMap<usize,usize>) -> NarrowTyping<'a> {
        NarrowTyping {
            bt, broad, block_index,
            position: None,
            possible: EquivalenceMap::new(|new: &mut Vec<AtomicTypeSpec>, old| {
                let old_set = old.iter().collect::<HashSet<_>>();
                *new = new.drain(..).filter(|v| old_set.contains(v)).collect::<Vec<_>>();
                if new.len() == 0 {
                    return Err("type mismatch/D".to_string());
                }
                Ok(())
            })
        }
    }

    fn error_at(&self, msg: &str) -> String {
        self.position.as_ref().map(|(file,line)|
            at(msg,Some((file.as_ref(),*line)))
        ).unwrap_or("*anon*".to_string())
    }

    fn is_seq(&self, reg: usize) -> bool {
        match self.broad.get(&reg).expect("missing register during typing") {
            BroadType::Atomic(_) => false,
            BroadType::Sequence => true,
        }
    }

    fn make_type(&self, spec: &TypeSpec, wilds: &HashMap<String,NarrowType>) -> Result<Option<NarrowType>,String> {
        Ok(match spec {
            TypeSpec::Atomic(a) => Some(NarrowType::Atomic(a.clone())),
            TypeSpec::Sequence(s) => Some(NarrowType::Sequence(s.clone())),
            TypeSpec::Wildcard(w) => { wilds.get(w).cloned() },
            TypeSpec::SequenceWildcard(w) => {
                wilds.get(w).map(|t| t.atomic_to_sequence()).transpose()?
            }
        })
    }

    fn make_wildcards(&mut self, block: &CodeBlock, args: &[usize]) -> Result<HashMap<String,WildcardType>,String> {
        let mut wilds = HashMap::new();
        for (spec,reg) in block.arguments.iter().zip(args.iter()) {
            let reg_broad = self.broad.get(reg).expect("missing broad type for register");
            match (&spec.arg_type,reg_broad) {
                (TypeSpec::Atomic(a), BroadType::Atomic(b)) if a == b => {},
                (TypeSpec::Sequence(_), BroadType::Sequence) => {}
                (TypeSpec::Wildcard(w), bt) => {
                    wilds.entry(w.to_string()).or_insert(WildcardType::Any).broad(bt)?;
                },
                (TypeSpec::SequenceWildcard(w), BroadType::Sequence) => {
                    wilds.entry(w.to_string()).or_insert(WildcardType::Any).atomic()?;
                },
                _ => {
                    return Err("type mismatch/E".to_string());
                }
            }
        }
        Ok(wilds)
    }

    fn apply_wildcard(&mut self, spec: &TypeSpec, wilds: &mut HashMap<String,WildcardType>) -> Result<Option<Vec<AtomicTypeSpec>>,String> {
        Ok(match spec {
            TypeSpec::Atomic(_) => None,
            TypeSpec::Sequence(s) => {
                Some(vec![s.clone()])
            },
            TypeSpec::Wildcard(w) => {
                let wild = wilds.entry(w.to_string()).or_insert(WildcardType::Any);
                match wild {
                    WildcardType::Sequence(s) => Some(vec![s.clone()]),
                    _ => None
                }
            },
            TypeSpec::SequenceWildcard(w) => {
                let wild = wilds.entry(w.to_string()).or_insert(WildcardType::Any);
                let atom = match wild {
                    WildcardType::Atomic(a) => Some(a.clone()),
                    WildcardType::AnyAtomic => None,
                    WildcardType::Any => None,
                    _ => {
                        return Err("type mismatch/F".to_string());
                    },
                };
                atom.map(|x| vec![x])
            },
        })
    }

    // XXX rets both wildcarded same but unrestricted
    fn code(&mut self, call: usize, name: usize, rets: &[usize], args: &[usize]) -> Result<(),String> {
        let block_index = *self.block_index.get(&call).expect("missing call block");
        let block = match self.bt.get_by_index(name)? {
            BTTopDefn::Code(c) => c.get_block(block_index),
            _ => { panic!("didn't get code with code index"); }
        };
        let mut wilds = self.make_wildcards(block,args)?;
        let mut ties = HashMap::new();
        /* process arguments */
        for (spec,reg) in block.arguments.iter().zip(args.iter()) {
            match &spec.arg_type {
                TypeSpec::Sequence(s) => {
                    self.possible.set(*reg,vec![s.clone()])?;
                },
                TypeSpec::SequenceWildcard(w) => {
                    let wild = wilds.get(w).expect("wildcard missed during generation");
                    if let Some(atomic) = wild.restricted_atomic()? {
                        self.possible.set(*reg,vec![atomic.clone()])?;
                    }
                    ties.entry((true,w)).or_insert(vec![]).push(*reg);
                },
                TypeSpec::Wildcard(w) => {
                    ties.entry((false,w)).or_insert(vec![]).push(*reg);
                }
                _ => {}
            }
        }
        /* process returns */
        for (spec,reg) in block.results.iter().zip(rets.iter()) {
            if let Some(restrs) = self.apply_wildcard(&spec.arg_type,&mut wilds)? {
                self.possible.set(*reg,restrs)?;
            }
            match &spec.arg_type {
                TypeSpec::Wildcard(w) => {
                    ties.entry((false,w)).or_insert(vec![]).push(*reg);
                },
                TypeSpec::SequenceWildcard(w) => {
                    ties.entry((true,w)).or_insert(vec![]).push(*reg);
                },
                _ => {}
            }
        }
        /* tie sequences with matching wilds. 
         * ?X is tied to ?X; seq(?X) to seq(?X).
         * We don't need to associate ?X with seq(?X) as that means ?X is atomic and so known
         *   completely by its broad type, and so seq(?X) will have just been bound.
         */
        for regs in ties.values() {
            let reg1 = regs.first().unwrap();
            for reg in regs {
                self.possible.equiv(*reg1,*reg)?;
            }
        }
        Ok(())
    }

    fn typestmt(&mut self, reg: usize, restrs: &[TypeRestriction]) -> Result<(),String> {
        if !self.is_seq(reg) { return Ok(()); }
        let restrs = restrs.iter().map(|r| {
            match r {
                TypeRestriction::Atomic(_) => None,
                TypeRestriction::AnySequence => None,
                TypeRestriction::Sequence(s) => Some(s.clone()),
            }
        }).collect::<Option<Vec<_>>>();
        if let Some(restrs) = restrs {
            self.possible.set(reg,restrs)?;
        }
        Ok(())
    }

    fn add(&mut self, stmt: &LinearStatement) -> Result<(),String> {
        self.position = Some((stmt.file.clone(),stmt.line_no));
        match &stmt.value {
            LinearStatementValue::Code(call,name,rets,args,_) => { 
                self.code(*call,*name,rets,args)?;
            },
            LinearStatementValue::Type(reg,restrs) => {
                self.typestmt(*reg,&restrs)?;
            },
            LinearStatementValue::WildEquiv(regs) => {
                let reg1 = regs[0];
                for reg in regs {
                    self.possible.equiv(reg1,*reg)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn finalise(&mut self) -> Result<HashMap<usize,NarrowType>,String> {
        self.position = None;
        let mut seq_types = HashMap::new();
        for reg in self.possible.keys() {
            let mut types = self.possible.get(*reg).cloned().unwrap_or(vec![AtomicTypeSpec::Boolean]);
            types.sort();
            if types.len() == 0 { return Err(format!("type mismatch/G r{:?}",*reg)); }
            seq_types.insert(*reg,types.swap_remove(0));
        }
        self.broad.iter().map(|(reg,broad)| {
            let narrow = match broad {
                BroadType::Atomic(a) => NarrowType::Atomic(a.clone()),
                BroadType::Sequence => NarrowType::Sequence(
                    seq_types.get(&reg).cloned().ok_or_else(|| format!("type mismatch/H"))?
                )
            };
            Ok((*reg,narrow))
        }).collect::<Result<_,_>>()
    }
}

pub(crate) fn narrow_type(bt: &BuildTree, broad: &HashMap<usize,BroadType>, block_index: &HashMap<usize,usize>, stmts: &[LinearStatement]) -> Result<HashMap<usize,NarrowType>,String> {
    let mut typing = NarrowTyping::new(bt,broad,block_index);
    for stmt in stmts {
        typing.add(stmt).map_err(|e| typing.error_at(&e))?;
    }
    typing.finalise()
}
