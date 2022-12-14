use std::{collections::{HashMap, HashSet}, hash::Hash};

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

pub(crate) struct EquivalenceMap<K: PartialEq+Eq+Hash+Clone,V,E> {
    merge: Box<dyn Fn(&mut V,&V) -> Result<(),E>>,
    equiv: EquivalenceClass<K>,
    values: HashMap<K,V>,
    keys: HashSet<K>
}

impl<K: PartialEq+Eq+Hash+Clone,V,E> EquivalenceMap<K,V,E> {
    pub(crate) fn new<F>(merge: F) -> EquivalenceMap<K,V,E> where F: Fn(&mut V,&V) -> Result<(),E> + 'static {
        EquivalenceMap {
            merge: Box::new(merge),
            equiv: EquivalenceClass::new(),
            values: HashMap::new(),
            keys: HashSet::new()
        }
    }

    pub(crate) fn equiv(&mut self, a: K, b: K) -> Result<(),E> {
        self.keys.insert(a.clone());
        self.keys.insert(b.clone());
        let a = self.equiv.canon(a);
        let b = self.equiv.canon(b);
        self.equiv.equiv(a.clone(),b.clone());
        if a != b {
            let old_b  = self.values.remove(&b);
            match (self.values.get_mut(&a),old_b) {
                (None, Some(v)) => {
                    self.values.insert(a,v);
                },
                (Some(v1), Some(v2)) => {
                    (self.merge)(v1,&v2)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn set(&mut self, key: K, mut value: V) -> Result<(),E> {
        self.keys.insert(key.clone());
        let key = self.equiv.canon(key);
        if let Some(old) = self.values.get(&key) {
            (self.merge)(&mut value,old)?;
        }
        self.values.insert(key,value);
        Ok(())
    }

    pub(crate) fn get(&self, key: K) -> Option<&V> {
        self.values.get(&self.equiv.canon(key))
    }

    pub(crate) fn keys(&self) -> &HashSet<K> {
        &self.keys()
    }
}
