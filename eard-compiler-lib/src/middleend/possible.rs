use std::collections::{BTreeSet, HashSet};

use crate::model::checkstypes::{AtomicTypeSpec, TypeRestriction, intersect_restrictions, TypeSpec};

use super::narrowtyping::NarrowType;

#[derive(Clone,Debug)]
pub(crate) struct NarrowPoss {
    sequence: bool,
    atom: bool,
    number: bool,
    string: bool,
    boolean: bool,
    any_handle: bool,
    specific_handles: BTreeSet<String>,
    restrictions: Option<HashSet<TypeRestriction>>
}

impl NarrowPoss {
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
    pub(crate) fn calc_type(&self) -> Result<NarrowType,String> {
        for narrow in self.type_options() {
            if self.apply_restrs(&narrow) { return Ok(narrow); }
        }
        return Err(format!("cannot deduce type/A"));
    }

    pub(crate) fn any() -> NarrowPoss {
        NarrowPoss { 
            sequence: true, atom: true,
            number: true, string: true, boolean: true, any_handle: true, 
            specific_handles: BTreeSet::new(),
            restrictions: None
        }
    }

    fn none() -> NarrowPoss {
        NarrowPoss { 
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
        eprintln!("{:?}",self);
        if self.check_valid_bool() { Ok(()) } else { Err(format!("cannot deduce type/B")) }
    }

    pub(crate) fn atom_to_seq(&self) -> Result<NarrowPoss,String> {
        let mut out = self.clone();
        if !self.atom { return Err(format!("cannot deduce type")); }
        out.atom = false;
        out.sequence = true;
        if let Some(restr) = &self.restrictions {
            out.restrictions = Some(restr.iter().filter_map(|r| {
                match r {
                    TypeRestriction::Atomic(a) => Some(TypeRestriction::Sequence(a.clone())),
                    TypeRestriction::Sequence(_) => None,
                    TypeRestriction::AnySequence => None,
                    TypeRestriction::AnyAtomic => Some(TypeRestriction::AnySequence)
                }
            }).collect());
        }
        out.check_valid()?;
        Ok(out)
    }

    pub(crate) fn seq_to_atom(&self) -> Result<NarrowPoss,String> {
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
                    TypeRestriction::AnyAtomic => {},
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
                    TypeRestriction::AnySequence => self.sequence,
                    TypeRestriction::AnyAtomic => self.atom,
                }
            }).cloned().collect::<HashSet<_>>()
        });
    }

    pub(crate) fn atomic(&mut self) -> Result<(),String> {
        self.sequence = false;
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }

    pub(crate) fn sequence(&mut self) -> Result<(),String> {
        self.atom = false;
        self.filter_acceptable();
        self.check_valid()?;
        Ok(())
    }

    pub(crate) fn unify(&mut self, other: &NarrowPoss) -> Result<(),String> {
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

    pub(crate) fn restrict_by_type(&mut self, restrs: &[TypeRestriction]) -> Result<(),String> {
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

    fn restrict_by_spec_list(&mut self, specs: &[TypeSpec]) -> Result<(),String> {
        todo!();
    }

    pub(crate) fn restrict_by_spec(&mut self, seq: bool, spec: &AtomicTypeSpec) -> Result<(),String> {
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
