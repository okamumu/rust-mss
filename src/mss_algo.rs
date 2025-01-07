
use std::collections::HashMap;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

use dd::bdd::Node;
use dd::common::NodeId;
use dd::mtmdd2;
use dd::nodes::{DDForest, NonTerminal};
use dd::nodes::Terminal;
use dd::{mdd, mtmdd};

pub trait MDDValue : dd::common::TerminalNumberValue + From<i32> + FromStr {
    // fn from_i32(x: i32) -> Self {
    //     Self::from(x)
    // }
}

impl MDDValue for i64 {}
impl MDDValue for i32 {}

pub fn prob<V,T>(
    mdd: &mut mtmdd2::MtMdd2Manager<V>,
    node: &mtmdd2::Node,
    pv: &HashMap<String, Vec<T>>,
) -> HashMap<V, T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MDDValue,
{
    match node {
        mtmdd2::Node::Value(fnode) => {
            let mut cache = HashMap::new();
            vprob(&mut mdd.mtmdd_mut(), *fnode, &pv, &mut cache)
        }
        mtmdd2::Node::Bool(fnode) => {
            let mut cache = HashMap::new();
            bprob(&mut mdd.mdd_mut(), *fnode, &pv, &mut cache)
        }
    }
}

fn vprob<V,T>(
    mdd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    cache: &mut HashMap<NodeId, HashMap<V, T>>,
) -> HashMap<V, T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mtmdd::Node::Terminal(fnode) => {
            let mut map = HashMap::new();
            let value = fnode.value();
            map.insert(value, T::from(1.0));
            map
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let label = mdd.label(node).unwrap();
            let fp = pv.get(label).unwrap();
            let mut map = HashMap::new();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for (i, x) in fnodeid.into_iter().enumerate() {
                let tmp = vprob(mdd, x, pv, cache);
                for (k, v) in tmp.iter() {
                    let key = *k;
                    let value = *v;
                    let entry = map.entry(key).or_insert(T::from(0.0));
                    *entry = *entry + fp[i] * value;
                }
            }
            map
        }
        mtmdd::Node::Undet => HashMap::new(),
    };
    cache.insert(key, result.clone());
    result
}

fn bprob<V,T>(
    mdd: &mut mdd::MddManager,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    cache: &mut HashMap<NodeId, HashMap<V, T>>,
) -> HashMap<V, T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(node).unwrap() {
        mdd::Node::Zero => {
            let mut map = HashMap::new();
            let value = V::from(0);
            map.insert(value, T::from(1.0));
            map
        }
        mdd::Node::One => {
            let mut map = HashMap::new();
            let value = V::from(1);
            map.insert(value, T::from(1.0));
            map
        }
        mdd::Node::NonTerminal(fnode) => {
            let label = mdd.label(node).unwrap();
            let fp = pv.get(label).unwrap();
            let mut map = HashMap::new();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for (i, x) in fnodeid.into_iter().enumerate() {
                let tmp = bprob(mdd, x, pv, cache);
                for (k, v) in tmp.iter() {
                    let key = *k;
                    let value = *v;
                    let entry = map.entry(key).or_insert(T::from(0.0));
                    *entry = *entry + fp[i] * value;
                }
            }
            map
        }
        mdd::Node::Undet => HashMap::new(),
    };
    cache.insert(key, result.clone());
    result
}

pub fn minsol<V>(
    mdd: &mut mtmdd2::MtMdd2Manager<V>,
    node: &mtmdd2::Node,
) -> mtmdd2::Node
where
    V: MDDValue,
{
    match node {
        mtmdd2::Node::Value(fnode) => {
            let mut cache1 = HashMap::new();
            let mut cache2 = HashMap::new();
            let result = vminsol(&mut mdd.mtmdd_mut(), *fnode, &mut cache1, &mut cache2);
            mtmdd2::Node::Value(result)
        }
        mtmdd2::Node::Bool(fnode) => {
            let mut cache1 = HashMap::new();
            let mut cache2 = HashMap::new();
            let result = bminsol(&mut mdd.mdd_mut(), *fnode, &mut cache1, &mut cache2);
            mtmdd2::Node::Bool(result)
        }
    }
}

fn vminsol<V>(
    dd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    cache1: &mut HashMap<NodeId, NodeId>,
    cache2: &mut HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId
where
    V: MDDValue,
{
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(node).unwrap() {
        mtmdd::Node::Terminal(fnode) => node,
        mtmdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let mut result = Vec::new();
            for (i, x) in fnodeid.iter().enumerate() {
                if i == 0 {
                    let tmp = vminsol(dd, *x, cache1, cache2);
                    result.push(tmp);
                } else {
                    let tmp = vminsol(dd, *x, cache1, cache2);
                    let tmp2 = vwithout(dd, fnodeid[i - 1], tmp, cache2);
                    result.push(tmp2);
                }
            }
            dd.create_node(headerid, &result)
        }
        mtmdd::Node::Undet => dd.undet(),
    };
    cache1.insert(key, result);
    result
}

fn vwithout<V>(
    mdd: &mut mtmdd::MtMddManager<V>,
    f: NodeId,
    g: NodeId, // minsol tree
    cache: &mut HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId
where
    V: MDDValue,
{
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return *x;
    }
    let result = match (mdd.get_node(f).unwrap(), mdd.get_node(g).unwrap()) {
        (mtmdd::Node::Undet, _) => g,
        (_, mtmdd::Node::Undet) => mdd.undet(),
        (mtmdd::Node::Terminal(fnode), mtmdd::Node::Terminal(gnode)) => {
            if fnode.value() == gnode.value() {
                mdd.undet()
            } else {
                g
            }
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::Terminal(_)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid.into_iter().map(|x| vwithout(mdd, x, g, cache)).collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::Terminal(_), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid.into_iter().map(|x| vwithout(mdd, f, x, cache)).collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(gnode))
            if mdd.level(f) > mdd.level(g) =>
        {
            vwithout(mdd, fnode[0], g, cache)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(gnode))
            if mdd.level(f) < mdd.level(g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid.into_iter().map(|x| vwithout(mdd, f, x, cache)).collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .zip(gnodeid.into_iter())
                .map(|(f, g)| vwithout(mdd, f, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
    };
    cache.insert(key, result);
    result
}

fn bminsol(
    dd: &mut mdd::MddManager,
    node: NodeId,
    cache1: &mut HashMap<NodeId, NodeId>,
    cache2: &mut HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(node).unwrap() {
        mdd::Node::Zero => dd.undet(),
        mdd::Node::One => node,
        mdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let mut result = Vec::new();
            for (i, x) in fnodeid.iter().enumerate() {
                if i == 0 {
                    let tmp = bminsol(dd, *x, cache1, cache2);
                    result.push(tmp);
                } else {
                    let tmp = bminsol(dd, *x, cache1, cache2);
                    let tmp2 = bwithout(dd, fnodeid[i - 1], tmp, cache2);
                    result.push(tmp2);
                }
            }
            dd.create_node(headerid, &result)
        }
        mdd::Node::Undet => dd.undet(),
    };
    cache1.insert(key, result);
    result
}

fn bwithout(
    mdd: &mut mdd::MddManager,
    f: NodeId,
    g: NodeId, // minsol tree
    cache: &mut HashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return *x;
    }
    let result = match (mdd.get_node(f).unwrap(), mdd.get_node(g).unwrap()) {
        (mdd::Node::Undet, _) => g,
        (_, mdd::Node::Undet) => mdd.undet(),
        (mdd::Node::Zero, mdd::Node::One) => mdd.one(),
        (mdd::Node::Zero, _) => g,
        (_, mdd::Node::Zero) => mdd.undet(), // probably this case is inpossible
        (mdd::Node::One, _) => mdd.undet(),
        (mdd::Node::NonTerminal(fnode), mdd::Node::One) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid.into_iter().map(|x| bwithout(mdd, x, g, cache)).collect();
            mdd.create_node(headerid, &tmp)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(gnode))
            if mdd.level(f) > mdd.level(g) =>
        {
            bwithout(mdd, fnode[0], g, cache)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(gnode))
            if mdd.level(f) < mdd.level(g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid.into_iter().map(|x| bwithout(mdd, f, x, cache)).collect();
            mdd.create_node(headerid, &tmp)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .zip(gnodeid.into_iter())
                .map(|(f, g)| bwithout(mdd, f, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
    };
    cache.insert(key, result);
    result
}
