use crate::prelude::*;

pub fn mdd_count<V, T>(
    mdd: &mtmdd2::MtMdd2Manager<V>,
    node: &mtmdd2::Node,
    ss: &HashSet<V>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    match node {
        mtmdd2::Node::Value(fnode) => {
            let mut cache = BddHashMap::default();
            if let Some(level) = mdd.mtmdd().level(*fnode) {
                let (nheaders, _, _, _) = mdd.mtmdd().size();
                let mut level2headers = vec![0; nheaders];
                for hid in 0..nheaders {
                    if let Some(h) = mdd.mtmdd().get_header(hid) {
                        level2headers[h.level()] = hid;
                    }
                }
                vmdd_count(mdd.mtmdd(), *fnode, ss, &mut cache, Some(level), &level2headers)
            } else {
                T::from(1)
            }
        }
        mtmdd2::Node::Bool(fnode) => {
            let mut cache = BddHashMap::default();
            if let Some(level) = mdd.mdd().level(*fnode) {
                let (nheaders, _, _) = mdd.mdd().size();
                let mut level2headers = vec![0; nheaders];
                for hid in 0..nheaders {
                    if let Some(h) = mdd.mdd().get_header(hid) {
                        level2headers[h.level()] = hid;
                    }
                }
                bmdd_count(mdd.mdd(), *fnode, ss, &mut cache, Some(level), &level2headers)
            } else {
                T::from(1)
            }
        }
    }
}

fn vmdd_count<V, T>(
    mdd: &mtmdd::MtMddManager<V>,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<(NodeId, Option<usize>), T>,
    level: Option<usize>,
    level2headers: &[usize],
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    let key = (node, level);
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mtmdd::Node::NonTerminal(_) | mtmdd::Node::Terminal(_) if level > mdd.level(node) => {
            let nedges = mdd.get_header(level2headers[level.unwrap()]).unwrap().edge_num() as u32;
            T::from(nedges) * vmdd_count(mdd, node, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers)
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = vmdd_count(mdd, x, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers);
                result = result + tmp;
            }
            result
        }
        mtmdd::Node::Terminal(fnode) => {
            let value = fnode.value();
            if ss.contains(&value) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mtmdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

fn bmdd_count<V, T>(
    mdd: &mdd::MddManager,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
    level: Option<usize>,
    level2headers: &[usize],
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mdd::Node::NonTerminal(_) | mdd::Node::Zero | mdd::Node::One if level > mdd.level(node) => {
            let nedges = mdd.get_header(level2headers[level.unwrap()]).unwrap().edge_num() as u32;
            T::from(nedges) * bmdd_count(mdd, node, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers)
        }
        mdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = bmdd_count(mdd, x, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers);
                result = result + tmp;
            }
            result
        }
        mdd::Node::Zero => {
            if ss.contains(&V::from(0)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::One => {
            if ss.contains(&V::from(1)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

pub fn zmdd_count<V, T>(
    mdd: &mtmdd2::MtMdd2Manager<V>,
    node: &mtmdd2::Node,
    ss: &HashSet<V>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    match node {
        mtmdd2::Node::Value(fnode) => {
            let mut cache = BddHashMap::default();
            vzmdd_count(mdd.mtmdd(), *fnode, ss, &mut cache)
        }
        mtmdd2::Node::Bool(fnode) => {
            let mut cache = BddHashMap::default();
            bzmdd_count(mdd.mdd(), *fnode, ss, &mut cache)
        }
    }
}

fn vzmdd_count<V, T>(
    mdd: &mtmdd::MtMddManager<V>,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mtmdd::Node::Terminal(fnode) => {
            let value = fnode.value();
            if ss.contains(&value) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = vzmdd_count(mdd, x, ss, cache);
                result = result + tmp;
            }
            result
        }
        mtmdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

fn bzmdd_count<V, T>(
    mdd: &mdd::MddManager,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mdd::Node::Zero => {
            if ss.contains(&V::from(0)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::One => {
            if ss.contains(&V::from(1)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = bzmdd_count(mdd, x, ss, cache);
                result = result + tmp;
            }
            result
        }
        mdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mdd() -> (mtmdd2::Node, mtmdd2::MtMdd2Manager<i32>) {
        let mut mgr = mtmdd2::MtMdd2Manager::<i32>::new(); 
        let h = mgr.create_header(0, "x", 3);
        let zero = mgr.value(0);
        let one = mgr.value(1);
        let two = mgr.value(2);
        let x = mgr.create_node(h, &vec![zero, one, two]);
        let h = mgr.create_header(1, "y", 3);
        let y = mgr.create_node(h, &vec![zero, one, two]);
        let h = mgr.create_header(2, "z", 3);
        let z = mgr.create_node(h, &vec![zero, one, two]);
        let tmp = mgr.add(x, y);
        (mgr.mul(tmp, z), mgr)
    }

    #[test]
    fn test_zmdd_count() {
        let (node, mut mgr) = create_mdd();
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        println!("{}", mgr.dot_string(node.clone()));
        let result: u64 = zmdd_count(&mut mgr, &node, &ss);
        println!("{}", result);
        assert!(result == 3);
    }

    #[test]
    fn test_mdd_count() {
        let (node, mut mgr) = create_mdd();
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        println!("{}", mgr.dot_string(node.clone()));
        let result: u64 = mdd_count(&mut mgr, &node, &ss);
        println!("{}", result);
        assert!(result == 11);
    }
}