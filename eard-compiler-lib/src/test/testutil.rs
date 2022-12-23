use std::fmt;

#[cfg(test)]
use std::{collections::HashMap};

#[cfg(test)]
pub(crate) struct AllocDumper {
    next_call: usize,
    seen: HashMap<usize,usize>
}

#[cfg(test)]
impl AllocDumper {
    pub(crate) fn new() -> AllocDumper {
        AllocDumper { next_call: 0, seen: HashMap::new() }
    }

    pub(crate) fn get(&mut self, input: usize) -> usize {
        let (seen,next_call) = (&mut self.seen,&mut self.next_call);
        *seen.entry(input).or_insert_with(|| {
            *next_call +=1;
            *next_call
        })
    }
}

pub(crate) fn sepfmt<X>(input: &mut dyn Iterator<Item=X>, sep: &str, prefix: &str) -> String where X: fmt::Debug {
    input.map(|x| format!("{}{:?}",prefix,x)).collect::<Vec<_>>().join(sep)
}
