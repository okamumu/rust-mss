use crate::prelude::*;

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
    ss: &[bool],
    node: NodeId,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        bdd::Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        bdd::Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        bdd::Node::NonTerminal(fnode) => {
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
        bdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

pub fn zdd_count<T>(
    dd: &BddManager,
    ss: &[bool],
    node: NodeId,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(node).unwrap() {
        bdd::Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        bdd::Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        bdd::Node::NonTerminal(fnode) => {
            zdd_count(dd, ss, fnode[0], cache) + zdd_count(dd, ss, fnode[1], cache)
        }
        bdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

