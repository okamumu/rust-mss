use std::collections::HashMap;
use std::ops::{Add, Sub, Mul};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use dd::dot::Dot;
use dd::mtmdd2;

use crate::mss_algo;



pub struct MddMgr<V> {
    mdd: Rc<RefCell<mtmdd2::MtMdd2Manager<V>>>,
    vars: HashMap<String, MddNode<V>>,
}

#[derive(Clone, Debug)]
pub struct MddNode<V> {
    parent: Weak<RefCell<mtmdd2::MtMdd2Manager<V>>>,
    node: mtmdd2::Node,
}

impl<V> MddNode<V> where V: mss_algo::MDDValue {
    fn new(parent: &Rc<RefCell<mtmdd2::MtMdd2Manager<V>>>, node: mtmdd2::Node) -> Self {
        MddNode {
            parent: Rc::downgrade(&parent),
            node,
        }
    }
}

impl<V> MddMgr<V> where V: mss_algo::MDDValue {
    pub fn new() -> Self {
        MddMgr {
            mdd: Rc::new(RefCell::new(mtmdd2::MtMdd2Manager::new())),
            vars: HashMap::new(),
        }
    }

    pub fn size(&self) -> (usize, usize, usize, usize) {
        self.mdd.borrow().size()
    }

    pub fn zero(&self) -> MddNode<V> {
        MddNode::new(&self.mdd, self.mdd.borrow().zero())
    }

    pub fn one(&self) -> MddNode<V> {
        MddNode::new(&self.mdd, self.mdd.borrow().one())
    }

    pub fn val(&self, value: V) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let node = mdd.value(value);
        MddNode::new(&self.mdd, node)
    }

    pub fn defvar(&mut self, label: &str, range: usize) -> MddNode<V> {
        if let Some(node) = self.vars.get(label) {
            return node.clone();
        } else {
            let level = self.vars.len();
            let result = {
                let mut mdd = self.mdd.borrow_mut();
                let range_: Vec<_> = (0..range).map(|x| V::from(x as i32)).collect(); // TODO: it should be changed in gen_var
                let node = mtmdd2::gen_var(&mut mdd, label, level, &range_);
                MddNode::new(&self.mdd, node)
            };
            self.vars.insert(label.to_string(), result.clone());
            result
        }
    }

    pub fn var(&self, label: &str) -> Option<MddNode<V>> {
        if let Some(node) = self.vars.get(label) {
            Some(node.clone())
        } else {
            None
        }
    }

    pub fn rpn(&mut self, rpn: &str, vars: &HashMap<String, usize>) -> Result<MddNode<V>, &'static str> {
        let tokens: Vec<_> = rpn
            .split_whitespace()
            .map(|x| {
                match x {
                    "+" => mtmdd2::Token::Add,
                    "-" => mtmdd2::Token::Sub,
                    "*" => mtmdd2::Token::Mul,
                    "/" => mtmdd2::Token::Div,
                    "==" => mtmdd2::Token::Eq,
                    "!=" => mtmdd2::Token::Neq,
                    "<" => mtmdd2::Token::Lt,
                    "<=" => mtmdd2::Token::Lte,
                    ">" => mtmdd2::Token::Gt,
                    ">=" => mtmdd2::Token::Gte,
                    "&&" => mtmdd2::Token::And,
                    "||" => mtmdd2::Token::Or,
                    "!" => mtmdd2::Token::Not,
                    "?" => mtmdd2::Token::IfElse,
                    "True" => {
                        let node = {
                            let mdd = self.mdd.borrow();
                            mdd.one()
                        };
                        mtmdd2::Token::Value(node)
                    }
                    "False" => {
                        let node = {
                            let mdd = self.mdd.borrow();
                            mdd.zero()
                        };
                        mtmdd2::Token::Value(node)
                    }
                    _ => {
                        // parse whether it is a number or a variable
                        match x.parse::<V>() {
                            Ok(val) => {
                                let node = {
                                    let mut mdd = self.mdd.borrow_mut();
                                    mdd.value(val)
                                };
                                mtmdd2::Token::Value(node)
                            }
                            Err(_) => {
                                let result = self.vars.get(x);
                                if let Some(node) = result {
                                    mtmdd2::Token::Value(node.node.clone())
                                } else {
                                    match vars.get(x) {
                                        Some(range) => {
                                            let node = self.defvar(x, range.clone());
                                            mtmdd2::Token::Value(node.node.clone())
                                        }
                                        None => panic!("Unknown variable: {}", x),
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .collect();
        let mut mdd = self.mdd.borrow_mut();
        if let Ok(node) = mtmdd2::build_from_rpn(&mut mdd, &tokens) {
            Ok(MddNode::new(&self.mdd, node))
        } else {
            Err("Invalid expression")
        }
    }

    pub fn and(&mut self, nodes: &Vec<MddNode<V>>) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = mdd.one();
        for node in xs {
            result = mdd.and(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    pub fn or(&mut self, nodes: &Vec<MddNode<V>>) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = mdd.zero();
        for node in xs {
            result = mdd.or(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    pub fn not(&mut self, node: &MddNode<V>) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let result = mdd.not(node.node);
        MddNode::new(&self.mdd, result)
    }

    pub fn ifelse(&mut self, cond: &MddNode<V>, then: &MddNode<V>, els: &MddNode<V>) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let result = mdd.ite(cond.node, then.node, els.node);
        MddNode::new(&self.mdd, result)
    }
}

impl<V> MddNode<V> where V: mss_algo::MDDValue {
    pub fn dot(&self) -> String {
        let mddmgr = self.parent.upgrade().unwrap();
        let mdd = mddmgr.borrow();
        mdd.dot_string(self.node)
    }

    pub fn add(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.add(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn sub(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.sub(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn mul(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.mul(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn div(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.div(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn eq(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.eq(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn ne(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.neq(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn lt(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.lt(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn le(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.lte(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn gt(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.gt(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn ge(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.gte(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn value(&self, other: V) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.value(other);
        MddNode::new(&mddmgr, node)
    }

    pub fn boolean(&self, other: bool) -> MddNode<V> {
        if other {
            let mddmgr = self.parent.upgrade().unwrap();
            let mdd = mddmgr.borrow();
            let node = mdd.one();
            MddNode::new(&mddmgr, node)
        } else {
            let mddmgr = self.parent.upgrade().unwrap();
            let mdd = mddmgr.borrow();
            let node = mdd.zero();
            MddNode::new(&mddmgr, node)
        }
    }

    pub fn prob<T>(&mut self, pv: &HashMap<String, Vec<T>>) -> HashMap<V, T> 
    where
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    {
        let mgr = self.parent.upgrade().unwrap();
        let mut mdd = mgr.borrow_mut();
        mss_algo::prob(&mut mdd, &self.node, pv)
    }

    pub fn mpvs(&mut self) -> MddNode<V> {
        let mgr = self.parent.upgrade().unwrap();
        let mut mdd = mgr.borrow_mut();
        let node = mss_algo::minsol(&mut mdd, &self.node);
        MddNode::new(&mgr, node)
    }

    pub fn count_set(&self) -> (u64, u64) {
        match &self.node {
            mtmdd2::Node::Value(x) => {
                let mgr = self.parent.upgrade().unwrap();
                let mdd = mgr.borrow();
                mdd.count(self.node)
            }
            mtmdd2::Node::Bool(x) => {
                let mgr = self.parent.upgrade().unwrap();
                let mdd = mgr.borrow();
                mdd.count(self.node)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdd_mgr() {
        let mut mgr: MddMgr<i32> = MddMgr::new();
        let x = mgr.defvar("x", 3);
        let y = mgr.defvar("y", 3);
        let z = mgr.defvar("z", 3);
        // let zero = mgr.zero();
        // let one = mgr.one();
        // let two = mgr.val(2);
        let mut vars = HashMap::new();
        // vars.insert("x".to_string(), 3);
        // vars.insert("y".to_string(), 3);
        // vars.insert("z".to_string(), 3);
        // println!("vars: {:?}", mgr.vars.borrow());
        let rpn = "x y z + *";
        if let Ok(node) = mgr.rpn(rpn, &vars) {
            println!("{}", node.dot());
        }
    }
}
