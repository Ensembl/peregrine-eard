use std::collections::{HashSet, HashMap};

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
            bundles: vec![]
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
}
