// mod ft

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::{Add, Mul, Sub};

use dd::bdd::*;
use dd::common;
use dd::common::NodeId;
use dd::nodes::{DDForest, NonTerminal};

pub trait ProbValue:
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Clone + Copy + PartialEq + From<f64>
{
}

impl ProbValue for f64 {}

pub fn prob<T>(
    dd: &BddManager,
    node: NodeId,
    pv: &HashMap<String, T>,
    ss: &HashSet<bool>,
    cache: &mut common::HashMap<NodeId, T>,
) -> T
where
    T: ProbValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        Node::One => {
            if ss.contains(&true) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        Node::NonTerminal(fnode) => {
            let x = dd.label(node).unwrap();
            let fp = *pv.get(x).unwrap_or(&T::from(0.0));
            let low = prob(dd, fnode[0], pv, ss, cache);
            let high = prob(dd, fnode[1], pv, ss, cache);
            (T::from(1.0) - fp) * low + fp * high
        }
        Node::Undet => panic!("Undetermined node"),
    };
    cache.insert(key, result);
    result
}

pub fn minsol(
    dd: &mut BddManager,
    node: NodeId,
    cache1: &mut common::HashMap<NodeId, NodeId>,
    cache2: &mut common::HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        Node::Zero => dd.zero(),
        Node::One => dd.one(),
        Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let f0 = fnode[0];
            let f1 = fnode[1];
            let tmp = minsol(dd, f1, cache1, cache2);
            let high = without(dd, tmp, f0, cache2);
            let low = minsol(dd, f0, cache1, cache2);
            dd.create_node(headerid, low, high)
        }
        Node::Undet => panic!("Undetermined node"),
    };
    cache1.insert(key, result);
    result
}

// enum BddStackValue<'a, 'b> {
//     Bdd2(&'a bdd::BddNode, &'a bdd::BddNode),
//     BddHeader((NodeId, NodeId), &'b dd::nodes::NodeHeader),
// }

// pub fn minsol_stack(
//     dd: &mut bdd::Bdd,
//     node: &bdd::BddNode,
//     cache1: &mut HashMap<NodeId, bdd::BddNode>,
//     cache2: &mut HashMap<(NodeId, NodeId), bdd::BddNode>,
// ) -> bdd::BddNode {
//     let mut next_stack = Vec::with_capacity(2048);
//     let mut result_stack = Vec::with_capacity(2048);
//     next_stack.push(node);
//     while let Some(x) = next_stack.pop() {
//         if let Some(result) = cache1.get(&x.id()) {
//             result_stack.push(result.clone());
//             continue;
//         }
//         match x {
//             bdd::BddNode::Zero => {
//                 let result = dd.zero();
//                 result_stack.push(result.clone());
//             }
//             bdd::BddNode::One => {
//                 let result = dd.one();
//                 result_stack.push(result.clone());
//             }
//             bdd::BddNode::NonTerminal(fnode) => {
//                 next_stack.push(&fnode[0]);
//                 next_stack.push(&fnode[1]);
//                 next_stack.push(fnode);
//             }
//         }
//     }
//     let key = node.id();
//     match cache1.get(&key) {
//         Some(x) => x.clone(),
//         None => {
//             let result = match node {
//                 bdd::BddNode::Zero => dd.zero(),
//                 bdd::BddNode::One => dd.one(),
//                 bdd::BddNode::NonTerminal(fnode) => {
//                     let tmp = minsol(dd, &fnode[1], cache1, cache2);
//                     let high = without_stack(dd, &tmp, &fnode[0], cache2);
//                     let low = minsol(dd, &fnode[0], cache1, cache2);
//                     dd.create_node(fnode.header(), &low, &high)
//                 }
//             };
//             cache1.insert(key, result.clone());
//             result
//         }
//     }
// }

// pub fn without_stack(
//     dd: &mut bdd::Bdd,
//     f: &bdd::BddNode, // minsol tree
//     g: &bdd::BddNode,
//     cache: &mut HashMap<(NodeId, NodeId), bdd::BddNode>,
// ) -> bdd::BddNode {
//     let mut next_stack = Vec::new(); //with_capacity(2048);
//     let mut result_stack = Vec::new(); //with_capacity(2048);
//     next_stack.push(BddStackValue::Bdd2(f, g));
//     while let Some(stackvalue) = next_stack.pop() {
//         match stackvalue {
//             BddStackValue::BddHeader(key, header) => {
//                 let high = result_stack.pop().unwrap();
//                 let low = result_stack.pop().unwrap();
//                 let result = dd.create_node(&header, &low, &high);
//                 cache.insert(key, result.clone());
//                 result_stack.push(result.clone());
//             }
//             BddStackValue::Bdd2(f, g) => {
//                 let key = (f.id(), g.id());
//                 if let Some(x) = cache.get(&key) {
//                     result_stack.push(x.clone());
//                     continue;
//                 }
//                 match (f, g) {
//                     (bdd::BddNode::Zero, _) => {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (_, bdd::BddNode::Zero) => {
//                         let result = f;
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (_, bdd::BddNode::One) => {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (bdd::BddNode::One, bdd::BddNode::NonTerminal(gnode)) => {
//                         next_stack.push(BddStackValue::BddHeader(key, gnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode[1]));
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode[0]));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.id() == gnode.id() =>
//                     {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.level() > gnode.level() =>
//                     {
//                         next_stack.push(BddStackValue::BddHeader(key, fnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(&fnode[1], g));
//                         next_stack.push(BddStackValue::Bdd2(&fnode[0], g));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.level() < gnode.level() =>
//                     {
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode[0]));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode)) => {
//                         next_stack.push(BddStackValue::BddHeader(key, fnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(&fnode[1], &gnode[1]));
//                         next_stack.push(BddStackValue::Bdd2(&fnode[0], &gnode[0]));
//                     }
//                 }
//             }
//         }
//     }
//     if let Some(node) = result_stack.pop() {
//         node.clone()
//     } else {
//         panic!("result stack is empty");
//     }
// }

fn without(
    dd: &mut BddManager,
    f: NodeId,
    g: NodeId,
    cache: &mut common::HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let node = match (dd.get_node(f).unwrap(), dd.get_node(g).unwrap()) {
        (Node::Zero, _) => dd.zero(),
        (_, Node::Zero) => f,
        (_, Node::One) => dd.zero(),
        (Node::One, Node::NonTerminal(gnode)) => {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let low = without(dd, f, gnodeid[0], cache);
            let high = without(dd, f, gnodeid[1], cache);
            dd.create_node(headerid, low, high)
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
            dd.zero()
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if dd.level(f) > dd.level(g) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let low = without(dd, fnodeid[0], g, cache);
            let high = without(dd, fnodeid[1], g, cache);
            dd.create_node(headerid, low, high)
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if dd.level(f) < dd.level(g) => {
            without(dd, f, gnode[0], cache)
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let low = without(dd, fnodeid[0], gnodeid[0], cache);
            let high = without(dd, fnodeid[1], gnodeid[1], cache);
            dd.create_node(headerid, low, high)
        }
        (Node::Undet, _) | (_, Node::Undet) => panic!("Undetermined node"),
    };
    cache.insert(key, node);
    node
}

pub fn zdd_count<T>(
    dd: &BddManager,
    ss: &HashSet<bool>,
    node: NodeId,
    cache: &mut common::HashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::NonTerminal(fnode) => {
            zdd_count(dd, ss, fnode[0], cache) + zdd_count(dd, ss, fnode[1], cache)
        }
        Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

fn power<T>(x: T, n: usize) -> T
where
    T: Mul<Output = T> + From<u32> + Clone,
{
    let mut result = T::from(1);
    for _ in 0..n {
        result = result * x.clone();
    }
    result
}

pub fn bdd_count<T>(
    dd: &BddManager,
    ss: &HashSet<bool>,
    node: NodeId,
    cache: &mut common::HashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let current_level = dd.level(node).unwrap();
            if let Some(next_level) = dd.level(fnode[0]) {
                result = result
                    + power(T::from(2), current_level - next_level - 1)
                        * bdd_count(dd, ss, fnode[0], cache);
            } else {
                result =
                    result + power(T::from(2), current_level) * bdd_count(dd, ss, fnode[0], cache);
            }
            if let Some(next_level) = dd.level(fnode[1]) {
                result = result
                    + power(T::from(2), current_level - next_level - 1)
                        * bdd_count(dd, ss, fnode[1], cache);
            } else {
                result =
                    result + power(T::from(2), current_level) * bdd_count(dd, ss, fnode[1], cache);
            }
            result
        }
        Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

pub fn bmeas<T>(
    dd: &BddManager,
    ss: &HashSet<bool>,
    node: NodeId,
    env: &HashMap<String, T>,
) -> HashMap<String, T>
where
    T: ProbValue,
{
    let sorted_nodes = topological_sort(dd, node);
    let mut gradcache = HashMap::new();
    let mut bddcache = common::HashMap::default();
    let mut gradevent = HashMap::new();
    gradcache.insert(node, T::from(1.0));
    for f in sorted_nodes {
        match dd.get_node(f).unwrap() {
            Node::Zero | Node::One | Node::Undet => (),
            Node::NonTerminal(fnode) => {
                let w = *gradcache.get(&fnode.id()).unwrap_or(&T::from(0.0));
                let x = dd.label(f).unwrap();
                let p = *env.get(x).unwrap_or(&T::from(0.0));
                let barp = T::from(1.0) - p;
                let result0 = if let Some(&val) = gradcache.get(&fnode[0]) {
                    val + w * barp
                } else {
                    w * barp
                };
                gradcache.insert(fnode[0], result0);
                let result1 = if let Some(&val) = gradcache.get(&fnode[1]) {
                    val + w * p
                } else {
                    w * p
                };
                gradcache.insert(fnode[1], result1);
                let p0 = prob(dd, fnode[0], env, ss, &mut bddcache);
                let p1 = prob(dd, fnode[1], env, ss, &mut bddcache);
                let resultv = if let Some(&val) = gradevent.get(x) {
                    val + w * (p1 - p0)
                } else {
                    w * (p1 - p0)
                };
                gradevent.insert(x.to_string(), resultv);
            }
        }
    }
    gradevent
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckedState {
    Persistent,
    Temporary,
}

fn topological_sort(dd: &BddManager, f: NodeId) -> Vec<NodeId> {
    let mut result = Vec::new();
    let mut check = common::HashMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(f.clone());
    while let Some(node) = queue.pop_front() {
        match check.get(&node) {
            Some(&CheckedState::Temporary) => panic!("DAG has a closed path"),
            Some(&CheckedState::Persistent) => (),
            None => {
                visit(dd, node, &mut check, &mut result, &mut queue);
            }
        }
    }
    result.reverse();
    result
}

fn visit(
    dd: &BddManager,
    x: NodeId,
    check: &mut common::HashMap<NodeId, CheckedState>,
    result: &mut Vec<NodeId>,
    queue: &mut VecDeque<NodeId>,
) {
    match check.get(&x) {
        Some(&CheckedState::Temporary) => panic!("DAG has a closed path"),
        Some(&CheckedState::Persistent) => (),
        None => {
            check.insert(x, CheckedState::Temporary);
            match dd.get_node(x).unwrap() {
                Node::Zero | Node::One | Node::Undet => (),
                Node::NonTerminal(fnode) => {
                    for &m in fnode.iter() {
                        queue.push_back(m);
                        visit(dd, m, check, result, queue);
                    }
                }
            }
            check.insert(x, CheckedState::Persistent);
            result.push(x);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashSet;

//     use super::*;

//     #[test]
//     fn test_topological_sort() {
//         let mut dd = crate::bdd::BddMgr::new();
//         let x = dd.defvar("x");
//         let y = dd.defvar("y");
//         let z = dd.defvar("z");
//         let set = HashSet::new();
//         let f = dd.rpn("x y & z |", set).unwrap();
//         let tmp = f.node().clone();
//         let result = topological_sort(&tmp);

//         result.iter().for_each(|x| println!("{:?}", x));
//     }
// }
