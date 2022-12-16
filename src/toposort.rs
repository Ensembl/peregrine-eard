use std::{collections::HashMap, hash::Hash};

struct TopoNode<V> {
    incoming: Vec<usize>,
    incoming_build: Vec<usize>,
    outgoing: Vec<usize>,
    value: V,
    seen: bool
}

pub(crate) struct TopoSort<V: PartialEq+Eq+Hash+Clone> {
    nodes: Vec<TopoNode<V>>,
    lookup: HashMap<V,usize>,
    sort: Option<Vec<usize>>,
    limit: Option<u32>
}

impl<V: PartialEq+Eq+Hash+Clone> TopoSort<V> {
    pub(crate) fn new(limit: Option<u32>) -> TopoSort<V> {
        TopoSort {
            nodes: vec![],
            lookup: HashMap::new(),
            sort: None,
            limit
        }
    }

    fn order(&self) -> Option<Vec<&V>> {
        self.sort.as_ref().map(|sorting| {
            sorting.iter().map(|idx| {
                &self.nodes[*idx].value
            }).collect::<Vec<_>>()
        })
    }

    fn order_clone(&self) -> Option<Vec<V>> {
        self.sort.as_ref().map(|sorting| {
            sorting.iter().map(|idx| {
                self.nodes[*idx].value.clone()
            }).collect::<Vec<_>>()
        })
    }

    pub(crate) fn node(&mut self, value: V) {
        if self.lookup.contains_key(&value) { return; }
        let index = self.nodes.len();
        self.lookup.insert(value.clone(),index);
        self.nodes.push(TopoNode {
            incoming: vec![],
            incoming_build: vec![],
            outgoing: vec![],
            value,
            seen: false
        });
    }

    fn try_to_reach(&self, at: usize, b: usize, credits: &mut Option<u32>) -> bool {
        if at == b { return true; }
        if let Some(cr) = credits {
            if *cr == 0 { return true; } // rightside failure: assume we can
            *cr -= 1;
        }
        for dst in &self.nodes[at].outgoing {
            if self.try_to_reach(*dst,b,credits) { return true; }
        }
        false
    }

    fn test_arc(&mut self, a: usize, b: usize) -> bool {
        let mut credits = self.limit;
        if self.try_to_reach(b,a,&mut credits) { return false; }
        true
    }

    pub(crate) fn arc(&mut self, a: &V, b: &V) -> bool {
        if let (Some(a),Some(b)) = (self.lookup.get(a).cloned(),self.lookup.get(b).cloned()) {
            if a == b { return false; }
            if self.sort.is_some() && !self.test_arc(a,b) {
                return false;
            }
            self.nodes[a].outgoing.push(b);
            self.nodes[b].incoming.push(a);
        }
        true
    }

    pub(crate) fn sort(&mut self) -> bool {
        let mut ongoing = vec![];
        let mut sorted = vec![];
        for (i,node) in self.nodes.iter_mut().enumerate() {
            if node.incoming.len() == 0 {
                ongoing.push(i);
                sorted.push(i);
                node.seen = true;
            } else {
                node.seen = false;
            }
            node.incoming_build = node.incoming.clone();
        }
        let mut target = vec![];
        while let Some(src) = ongoing.pop() {
            target.clear();
            for dst in &self.nodes[src].outgoing {
                let src_idx = self.nodes[*dst].incoming_build.iter().position(|x| *x==src).unwrap();
                target.push((*dst,src_idx));
            }
            for (node_idx,link_idx) in target.drain(..) {
                let node = &mut self.nodes[node_idx];
                if !node.seen {
                    let incoming = &mut node.incoming_build;
                    incoming.swap_remove(link_idx);
                    if incoming.len() == 0 {
                        ongoing.push(node_idx);
                        sorted.push(node_idx);
                        node.seen = true;
                    }
                }
            }
        }
        if sorted.len() == self.nodes.len() {
            self.sort = Some(sorted);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topo_smoke() {
        let nodes = &[2,3,5,7,8,9,10,11];
        let from = &[5,7,7,3,3,11,11,11,8];
        let to = &[11,11,8,8,10,2,9,10,9];
        let mut topo = TopoSort::new(None);
        for node in nodes {
            topo.node(*node);
        }
        for (from,to) in from.iter().zip(to.iter()) {
            assert!(topo.arc(from,to));
        }
        topo.sort();
        assert_eq!(Some(vec![3,5,7,11,2,8,10,9]),topo.order_clone());
        assert!(topo.arc(&7,&3));
        assert!(topo.sort());
        assert_eq!(Some(vec![5,7,3,8,11,2,9,10]),topo.order_clone());
        assert!(topo.arc(&2,&3));
        assert!(topo.sort());
        assert_eq!(Some(vec![5,7,11,2,3,8,10,9]),topo.order_clone());
        assert!(!topo.arc(&8,&2));
        assert_eq!(Some(vec![5,7,11,2,3,8,10,9]),topo.order_clone());
        assert!(topo.arc(&7,&5));
        assert!(topo.sort());
        assert_eq!(Some(vec![7,5,11,2,3,8,10,9]),topo.order_clone());
    }

    #[test]
    fn topo_credit() {
        for limit in &[10,30] {
            let mut topo = TopoSort::new(Some(*limit));
            for i in 0..20 {
                topo.node(i);
            }
            for i in 0..19 {
                topo.arc(&i,&(i+1));
            }
            topo.sort();
            assert!(topo.arc(&16,&18));
            topo.sort();
            assert_eq!(topo.arc(&2,&4),*limit>10);
        }
    }
}
