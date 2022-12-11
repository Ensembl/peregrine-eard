use std::collections::{HashSet, HashMap};

use crate::model::Variable;

#[derive(Clone)]
pub(super) struct Bundle {
    name: String,
    used: HashSet<String>
}

impl Bundle {
    fn new(name: &str) -> Bundle {
        Bundle {
            name: name.to_string(),
            used: HashSet::new()
        }
    }

    pub(super) fn get_used(&self) -> &HashSet<String> { &self.used }
}

pub(super) struct BundleNamespace {
    bundles: Vec<HashMap<String,Bundle>>
}

impl BundleNamespace {
    pub(super) fn new() -> BundleNamespace {
        BundleNamespace {
            bundles: vec![HashMap::new()]
        }
    }

    pub(super) fn push(&mut self) {
        self.bundles.push(HashMap::new());
    }

    pub(super) fn pop(&mut self) {
        self.bundles.pop();
    }

    pub(super) fn get(&self, prefix: &str) -> Option<&Bundle> {
        if let Some(top) = self.bundles.last() {
            if let Some(bundle) = top.get(prefix) {
                return Some(bundle);
            }
        }
        None
    }

    pub(super) fn clear(&mut self, prefix: &str) {
        if let Some(top) = self.bundles.last_mut() {
            if let Some(bundle) = top.get_mut(prefix) {
                bundle.used.clear();
            }
        }
    }

    pub(super) fn remove(&mut self, prefix: &str, name: &str) {
        if let Some(top) = self.bundles.last_mut() {
            if let Some(bundle) = top.get_mut(prefix) {
                bundle.used.remove(name);
            }
        }
    }

    pub(super) fn add(&mut self, prefix: &str, name: &str) {
        if let Some(top) = self.bundles.last_mut() {
            top.entry(prefix.to_string()).or_insert_with(|| Bundle::new(name)).used.insert(name.to_string());
        }
    }

    pub(super) fn add_all(&mut self, to: &str, from: &HashSet<String>) {
        if let Some(top) = self.bundles.last_mut() {
            let to = top.entry(to.to_string()).or_insert_with(|| Bundle::new(to));
            to.used.extend(&mut from.iter().cloned());
        }
    }

    pub(super) fn merge(&mut self, to: &str, from: &str) {
        if let Some(top) = self.bundles.last_mut() {
            let mut from = top.get(from).cloned().unwrap_or(Bundle::new(from));
            let to = top.entry(to.to_string()).or_insert_with(|| Bundle::new(to));
            to.used.extend(&mut from.used.drain());
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub(crate) enum Position {
    Arg(usize),
    Return(usize),
    Repeater
}

pub(super) struct Transits {
    transits: HashMap<(Vec<usize>,Position),HashSet<String>>,
    call_stack: Vec<usize>
}

impl Transits {
    pub(super) fn new() -> Transits {
        Transits {
            transits: HashMap::new(),
            call_stack: vec![]
        }
    }

    pub(super) fn add(&mut self, position: Position, values: HashSet<String>) {
        self.transits.insert((self.call_stack.clone(),position),values);
    }

    pub(super) fn push(&mut self, call: usize) {
        self.call_stack.push(call);
    }

    pub(super) fn pop(&mut self) {
        self.call_stack.pop();
    }

    pub(super) fn take(mut self) -> HashMap<(Vec<usize>,Position),Vec<String>> {
        self.transits.drain().map(|(k,mut v)|  {
            let mut out = v.drain().collect::<Vec<_>>();
            out.sort(); // important to be sorted so as stable, as sequence is used in linearization
            (k,out)
        }).collect::<HashMap<_,_>>()
    }
}

struct VarRegisterLevel {
    regs: HashMap<Variable,usize>,
    prefixes: HashSet<String>
}

impl VarRegisterLevel {
    fn new() -> VarRegisterLevel {
        VarRegisterLevel { regs: HashMap::new(), prefixes: HashSet::new() }
    }

    fn add(&mut self, variable: &Variable, reg: usize) {
        self.regs.insert(variable.clone(),reg);
        if let Some(prefix) = &variable.prefix {
            self.prefixes.insert(prefix.to_string());
        }
    }

    fn get(&self, variable: &Variable) -> Result<usize,String> {
//        self.regs.get(variable).cloned().ok_or_else(|| format!("unknown variable '{}'",variable))
        Ok(self.regs.get(variable).cloned().expect(&format!("unknown variable '{}'",variable)))

    }

    fn check_used(&self, prefix: &str) -> bool {
        self.prefixes.contains(prefix)
    }
}

pub(super) struct VarRegisters {
    regs: Vec<VarRegisterLevel>
}

impl VarRegisters {
    pub(super) fn new() -> VarRegisters {
        VarRegisters {
            regs: vec![VarRegisterLevel::new()]
        }
    }

    pub(super) fn push(&mut self) {
        self.regs.push(VarRegisterLevel::new());
    }

    pub(super) fn pop(&mut self) {
        self.regs.pop();
    }

    fn top(&self) -> &VarRegisterLevel {
        self.regs.last().expect("empty name stack: should be impossible")
    }

    fn top_mut(&mut self) -> &mut VarRegisterLevel {
        self.regs.last_mut().expect("empty name stack: should be impossible")
    }

    pub(super) fn add(&mut self, variable: &Variable, reg: usize) {
        self.top_mut().add(variable,reg);
    }

    pub(super) fn get(&self, variable: &Variable) -> Result<usize,String> {
        self.top().get(variable)
    }

    pub(super) fn check_used(&self, prefix: &str) -> bool {
        self.top().check_used(prefix)
    }
}
