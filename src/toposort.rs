/* Consider deciding whether we can incrementally adding a link from A to B in a toposorted list.
 * 
 * If A<B we can add A to B immediately without reordering. If B<A, consider X, the sequence of
 * nodes suth that B<X<A and X is reachable from A, which is intermingled with Y which is the 
 * sequence B<Y<A where members of Y are not in X: B{X,Y}A. We show that we can reorder to XABY 
 * and maintain topological order iff B cannot reach A. For nodes M,N joined by an arc, for the 
 * following combinations we use the argument given in the following table, where L and R mean the 
 * left and right flank of the reordered region.
 * 
 * dest \ src  L  B  X  Y  A  R
 *   L         a  b  b  b  b  b
 *   B         c  a  b  b  b  b
 *   X         c  f  a  e  b  b
 *   Y         c  c  c  a  b  b
 *   A         c  f  c  d  a  b
 *   R         c  c  c  c  c  a
 * 
 * a. these are the same set and there is no reordering within a set
 * b. these are prohibited by the initial sort
 * c. these sets are ordered left to right in the final ordering
 * d. by construction, there are no links from Y to A
 * e. there are no links from Y to X, as A is reachable from X but not Y, a contradiction
 * f. requirement B cannot reach A, it can therefore not reach X (which can reach A)
 * 
 * So to add a new arc, we need to check whether we can reach A from B and build the sequence X 
 * which are those nodes which can reach A. This is essentially the same problem if we search for
 * A reachability [B,X). This is best achieved as a linear scan right to left: flags can be left
 * on members of X as we go. When we are at a node x, any outgoing links must be to the right and
 * all nodes to the right have been scanned, so we can just look at their flags.
 */

use std::{collections::{HashMap, VecDeque}, hash::Hash, fmt};

struct TopoNode<V> {
    incoming: Vec<usize>,
    incoming_build: Vec<usize>,
    outgoing: Vec<usize>,
    value: V,
    flag: u64
}

pub(crate) struct TopoSort<V: PartialEq+Eq+Hash+Clone> {
    nodes: Vec<TopoNode<V>>,
    lookup: HashMap<V,usize>,
    sort: Option<(Vec<usize>,Vec<usize>)>, // (pos to name,name to pos)
    next_flag: u64
}

impl<V: PartialEq+Eq+Hash+Clone+fmt::Debug> TopoSort<V> { // XXX Debug
    pub(crate) fn new() -> TopoSort<V> {
        TopoSort {
            nodes: vec![],
            lookup: HashMap::new(),
            sort: None,
            next_flag: 1
        }
    }

    pub(crate) fn order(&self) -> Option<Vec<&V>> {
        self.sort.as_ref().map(|(sorting,_)| {
            sorting.iter().map(|idx| {
                &self.nodes[*idx].value
            }).collect::<Vec<_>>()
        })
    }

    #[cfg(test)]
    pub(crate) fn order_clone(&self) -> Option<Vec<V>> {
        self.sort.as_ref().map(|(sorting,_)| {
            sorting.iter().map(|idx| {
                self.nodes[*idx].value.clone()
            }).collect::<Vec<_>>()
        })
    }

    pub(crate) fn distance(&self, a: &V, b: &V) -> Option<usize> {
        if let  (Some(a_name),Some(b_name),Some(name_to_pos)) = 
                (self.lookup.get(a),self.lookup.get(b),self.sort.as_ref().map(|x| &x.1)) {
            let (pos_a,pos_b) = (name_to_pos[*a_name],name_to_pos[*b_name]);
            if pos_a < pos_b { Some(0) } else { Some(pos_a-pos_b) }
        } else {
            None
        }
    }

    pub(crate) fn node(&mut self, value: V, after: Option<&V>) {
        if self.lookup.contains_key(&value) { return; }
        let name = self.nodes.len();
        self.lookup.insert(value.clone(),name);
        self.nodes.push(TopoNode {
            incoming: vec![],
            incoming_build: vec![],
            outgoing: vec![],
            value,
            flag: 0
        });
        if self.next_flag > 1 {
            let (pos_to_name,name_to_pos) = self.sort.as_mut().map(|x| (&mut x.0,&mut x.1)).unwrap();
            let mut at_pos = name;
            if let Some(after) = after {
                if let Some(after_name) = self.lookup.get(after) {
                    at_pos = name_to_pos[*after_name]+1;
                }
            }
            pos_to_name.insert(at_pos,name);
            *name_to_pos = name_to_pos.iter().map(|pos| {
                if *pos >= at_pos { pos+1 } else { *pos } 
            }).collect();
            name_to_pos.push(at_pos);
        }
    }

    fn reorder(&mut self, a: usize, b: usize) -> bool {
        let (pos_to_name,name_to_pos) = self.sort.as_mut().map(|x| (&mut x.0,&mut x.1)).unwrap();
        let our_flag = self.next_flag;
        self.next_flag += 1;
        let a_pos = name_to_pos[a];
        let b_pos = name_to_pos[b];
        if a_pos <= b_pos { return true; }
        /* B {X,Y} A. Flag X (and B) */
        self.nodes[pos_to_name[a_pos]].flag = our_flag;
        for pos in (b_pos..a_pos).rev() {
            let name = pos_to_name[pos];
            for dst_name in &self.nodes[name].outgoing {
                if self.nodes[*dst_name].flag == our_flag {
                    self.nodes[name].flag = our_flag;
                    break;
                }
            }
        }
        /* Abandon if B has been flagged */
        if self.nodes[pos_to_name[b_pos]].flag == our_flag {
            return false;
        }
        /* B[XY]A to XABY */
        let mut new_names = vec![];
        for pos in b_pos..a_pos+1 {
            let name = pos_to_name[pos];
            if self.nodes[name].flag == our_flag {
                new_names.push(name);
            }
        }
        for pos in b_pos..a_pos+1 {
            let name = pos_to_name[pos];
            if self.nodes[name].flag != our_flag {
                new_names.push(name);
            }
        }
        for (i,name) in new_names.iter().enumerate() {
            pos_to_name[b_pos+i] = *name;
            name_to_pos[*name] = b_pos+i;
        }
        true
    }

    pub(crate) fn arc(&mut self, a: &V, b: &V) -> bool {
        if let (Some(a),Some(b)) = (self.lookup.get(a).cloned(),self.lookup.get(b).cloned()) {
            if a == b { return false; }
            if self.sort.is_some() {
                if !self.reorder(a,b) { return false; }
            }
            self.nodes[a].outgoing.push(b);
            self.nodes[b].incoming.push(a);
        }
        true
    }

    pub(crate) fn sort(&mut self) -> bool {
        let our_flag = self.next_flag;
        self.next_flag += 1;
        let mut ongoing = VecDeque::new();
        let mut sorted = vec![];
        let mut roots = vec![];
        let mut rev_sorted = vec![0;self.nodes.len()];
        for (i,node) in self.nodes.iter_mut().enumerate() {
            if node.incoming.len() == 0 {
                roots.push(i);
            }
            node.incoming_build = node.incoming.clone();
        }
        roots.reverse();
        let mut target = vec![];
        while let Some(src) = ongoing.pop_front().or_else(|| roots.pop()) {
            rev_sorted[src] = sorted.len();
            sorted.push(src);
            target.clear();
            for dst in &self.nodes[src].outgoing {
                let src_idx = self.nodes[*dst].incoming_build.iter().position(|x| *x==src).unwrap();
                target.push((*dst,src_idx));
            }
            for (node_idx,link_idx) in target.drain(..) {
                let node = &mut self.nodes[node_idx];
                if !node.flag != our_flag {
                    let incoming = &mut node.incoming_build;
                    incoming.swap_remove(link_idx);
                    if incoming.len() == 0 {
                        ongoing.push_front(node_idx);
                        node.flag = our_flag;
                    }
                }
            }
        }
        if sorted.len() == self.nodes.len() {
            self.sort = Some((sorted,rev_sorted));
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
        let to  = &[11,11,8,8,10,2,9,10,9];    

        let mut topo = TopoSort::new();
        for node in nodes {
            topo.node(*node,None);
        }
        for (src,dst) in from.iter().zip(to.iter()) {
            assert!(topo.arc(src,dst));
        }
        topo.sort();
        assert_eq!(Some(vec![3,5,7,8,11,10,9,2]),topo.order_clone());
        assert!(topo.arc(&7,&3));
        assert_eq!(Some(vec![7,3,5,8,11,10,9,2]),topo.order_clone());
        assert!(topo.arc(&2,&3));
        assert_eq!(Some(vec![7,5,11,2,3,8,10,9]),topo.order_clone());
        assert!(!topo.arc(&8,&2));
        assert_eq!(Some(vec![7,5,11,2,3,8,10,9]),topo.order_clone());
        assert!(topo.arc(&7,&5));
        assert_eq!(Some(vec![7,5,11,2,3,8,10,9]),topo.order_clone());
        assert!(topo.arc(&9,&10));
        assert_eq!(Some(vec![7,5,11,2,3,8,9,10]),topo.order_clone());
    }

    #[test]
    fn topo_credit() {
        let mut topo = TopoSort::new();
        for i in 0..20 {
            topo.node(i,None);
        }
        for i in 0..19 {
            topo.arc(&i,&(i+1));
        }
        topo.sort();
        assert_eq!(0,topo.distance(&16,&18).unwrap());
        assert_eq!(2,topo.distance(&4,&2).unwrap());
    }
}
