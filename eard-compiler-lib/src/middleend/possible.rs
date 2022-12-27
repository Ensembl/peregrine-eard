use std::collections::{BTreeSet};
use crate::model::checkstypes::{AtomicTypeSpec, TypeSpec};
use super::{narrowtyping::NarrowType, broadtyping::BroadType};

#[derive(Clone,Debug)]
pub(crate) struct NarrowPoss {
    number: bool,
    string: bool,
    boolean: bool,
    any_handle: bool,
    specific_handles: BTreeSet<String>
}

impl NarrowPoss {
    pub(crate) fn from_type_specs(specs: &[TypeSpec]) -> NarrowPoss {
        let mut out = NarrowPoss::none();
        for spec in specs {
            let atom = match spec {
                TypeSpec::Atomic(a) => Some(a),
                TypeSpec::Sequence(a) => Some(a),
                _ => { return NarrowPoss::any() }
            };
            if let Some(atom) = atom {
                match atom {
                    AtomicTypeSpec::Number => { out.number = true; }
                    AtomicTypeSpec::String => { out.string = true; },
                    AtomicTypeSpec::Boolean => { out.boolean = true; },
                    AtomicTypeSpec::Handle(h) => { out.specific_handles.insert(h.to_string()); },
                }
            }
        }
        out
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

    fn type_options(&self, broad: &BroadType) -> Vec<NarrowType> {
        let mut out = vec![];
        let atoms = self.atom_type_options();
        match broad {
            BroadType::Atomic => {
                out.extend(atoms.iter().map(|a| NarrowType::Atomic(a.clone())));
            },
            BroadType::Sequence => {
                out.extend(atoms.iter().map(|a| NarrowType::Sequence(a.clone())));
            }
        }
        out
    }

    pub(crate) fn calc_type(&self, broad: &BroadType) -> Result<NarrowType,String> {
        for narrow in self.type_options(broad) {
            return Ok(narrow);
        }
        return Err(format!("cannot deduce type/A"));
    }

    pub(crate) fn any() -> NarrowPoss {
        NarrowPoss { 
            number: true, string: true, boolean: true, any_handle: true, 
            specific_handles: BTreeSet::new()
        }
    }

    fn none() -> NarrowPoss {
        NarrowPoss { 
            number: false, string: false, boolean: false, any_handle: false, 
            specific_handles: BTreeSet::new()
        }
    }

    fn check_valid_bool(&self) -> bool {
        self.number || self.string || self.boolean || self.any_handle || self.specific_handles.len() > 0
    }

    fn check_valid(&self) -> Result<(),String> {
        //eprintln!("{:?}",self);
        if self.check_valid_bool() { Ok(()) } else { Err(format!("cannot deduce type/B")) }
    }

    pub(crate) fn unify(&mut self, other: &NarrowPoss) -> Result<(),String> {
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
        self.check_valid()?;
        Ok(())
    }

    pub(crate) fn restrict_by_spec(&mut self, spec: &AtomicTypeSpec) -> Result<(),String> {
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
        *self = new;
        self.check_valid()?;
        Ok(())
    }
}
