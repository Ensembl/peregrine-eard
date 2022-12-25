use std::{collections::{HashMap}, hash::Hash};

#[derive(Debug)]
pub(crate) struct EquivalenceClass<X: PartialEq+Eq+Hash+Clone> {
    equiv: HashMap<X,X>
}

impl<X: PartialEq+Eq+Hash+Clone> EquivalenceClass<X> {
    pub(crate) fn new() -> EquivalenceClass<X> {
        EquivalenceClass {
            equiv: HashMap::new()
        }
    }

    pub(crate) fn canon(&self, mut value: X) -> X {
        while let Some(new_value) = self.equiv.get(&value) {
            value = new_value.clone();
        }
        value
    }

    /* a survives as new canon */
    pub(crate) fn equiv(&mut self, a: X, b: X) {
        let a = self.canon(a);
        let b = self.canon(b);
        if a != b {
            self.equiv.insert(b,a);
        }
    }

    pub(crate) fn build(&mut self) {
        let mut new = HashMap::new();
        for k in self.equiv.keys() {
            let v = self.canon(k.clone());
            new.insert(k.clone(),v);
        }
        self.equiv = new;
    }

    pub(crate) fn get<'a>(&'a self, k: &'a X) -> &'a X {
        self.equiv.get(k).unwrap_or_else(|| k)
    }
}
