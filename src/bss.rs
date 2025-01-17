//

use dd::bdd;
use dd::common;
use dd::common::HeaderId;
use dd::common::Level;
use dd::common::NodeId;
use dd::dot::Dot;
use dd::nodes::DDForest;
use std::collections::HashMap;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::rc::Weak;

use crate::bdd_path::BddPath;
use crate::bdd_path::ZddPath;
use crate::bss_algo;
use crate::bss_algo::ProbValue;

pub struct BssMgr {
    bdd: Rc<RefCell<bdd::BddManager>>,
    vars: HashMap<String, NodeId>,
}

#[derive(Clone)]
pub struct BddNode {
    parent: Weak<RefCell<bdd::BddManager>>,
    node: NodeId,
}

impl BddNode {
    pub fn new(bdd: &Rc<RefCell<bdd::BddManager>>, node: NodeId) -> Self {
        BddNode {
            parent: Rc::downgrade(bdd),
            node: node,
        }
    }
}

impl BssMgr {
    // constructor
    pub fn new() -> Self {
        BssMgr {
            bdd: Rc::new(RefCell::new(bdd::BddManager::new())),
            vars: HashMap::default(),
        }
    }

    // size
    pub fn size(&self) -> (usize, usize, usize) {
        self.bdd.borrow().size()
    }

    // zero
    pub fn zero(&self) -> BddNode {
        BddNode::new(&self.bdd, self.bdd.borrow().zero())
    }

    // one
    pub fn one(&self) -> BddNode {
        BddNode::new(&self.bdd, self.bdd.borrow().one())
    }

    pub fn create_node(&self, h: HeaderId, x0: &BddNode, x1: &BddNode) -> BddNode {
        let f0 = x0.node;
        let f1 = x1.node;
        BddNode::new(&self.bdd, self.bdd.borrow_mut().create_node(h, f0, f1))
    }

    // defvar
    pub fn defvar(&mut self, var: &str) -> BddNode {
        if let Some(node) = self.vars.get(var) {
            return BddNode::new(&self.bdd, *node);
        } else {
            let level = self.vars.len();
            let mut bdd = self.bdd.borrow_mut();
            let h = bdd.create_header(level, var);
            let x0 = bdd.zero();
            let x1 = bdd.one();
            let node = bdd.create_node(h, x0, x1);
            self.vars.insert(var.to_string(), node);
            BddNode::new(&self.bdd, node)
        }
    }

    pub fn get_varorder(&self) -> Vec<String> {
        let bdd = self.bdd.borrow();
        let mut result = vec!["?".to_string(); self.vars.len()];
        for (k, v) in self.vars.iter() {
            let node = bdd.get_node(*v).unwrap();
            let hid = node.headerid().unwrap();
            let header = bdd.get_header(hid).unwrap();
            result[header.level()] = k.clone();
        }
        result
    }

    pub fn rpn(&mut self, expr: &str) -> Result<BddNode, String> {
        let mut stack = Vec::new();
        let mut cache = HashMap::new();
        for token in expr.split_whitespace() {
            match token {
                "0" | "False" => {
                    let bdd = self.bdd.borrow_mut();
                    stack.push(bdd.zero());
                }
                "1" | "True" => {
                    let bdd = self.bdd.borrow_mut();
                    stack.push(bdd.one());
                }
                "&" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.and(left, right));
                }
                "|" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.or(left, right));
                }
                "^" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.xor(left, right));
                }
                "~" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let node = stack.pop().unwrap();
                    stack.push(bdd.not(node));
                }
                "?" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let else_ = stack.pop().unwrap();
                    let then = stack.pop().unwrap();
                    let cond = stack.pop().unwrap();
                    stack.push(bdd.ite(cond, then, else_));
                }
                _ if token.starts_with("save(") && token.ends_with(")") => {
                    let name = &token[5..token.len() - 1];
                    if let Some(node) = stack.last() {
                        cache.insert(name.to_string(), node.clone());
                    } else {
                        return Err("Stack is empty for save operation".to_string());
                    }
                }
                _ if token.starts_with("load(") && token.ends_with(")") => {
                    let name = &token[5..token.len() - 1];
                    if let Some(node) = cache.get(name) {
                        stack.push(node.clone());
                    } else {
                        return Err(format!("No cached value for {}", name));
                    }
                }
                _ => {
                    let node = self.defvar(token);
                    stack.push(node.node);
                }
            }
        }
        if stack.len() == 1 {
            return Ok(BddNode::new(&self.bdd, stack.pop().unwrap()));
        } else {
            return Err("Invalid expression".to_string());
        }
    }

    pub fn and(&self, nodes: &[BddNode]) -> BddNode {
        let mut bdd = self.bdd.borrow_mut();
        let mut result = self.one().node;
        for node in nodes {
            result = bdd.and(result, node.node);
        }
        BddNode::new(&self.bdd, result)
    }

    pub fn or(&self, nodes: &[BddNode]) -> BddNode {
        let mut bdd = self.bdd.borrow_mut();
        let mut result = self.zero().node;
        for node in nodes {
            result = bdd.or(result, node.node);
        }
        BddNode::new(&self.bdd, result)
    }

    // pub fn cache_clear(&self) {
    //     self.bdd.borrow_mut().cache_clear();
    // }
}

impl BddNode {
    pub fn get_mgr(&self) -> Rc<RefCell<bdd::BddManager>> {
        self.parent.upgrade().unwrap()
    }

    pub fn get_id(&self) -> NodeId {
        self.node
    }

    pub fn get_header(&self) -> Option<HeaderId> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node)?;
        node.headerid()
    }

    pub fn get_level(&self) -> Option<Level> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node)?;
        let hid = node.headerid()?;
        let header = bdd.get_header(hid)?;
        Some(header.level())
    }

    pub fn get_label(&self) -> Option<String> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node)?;
        let hid = node.headerid()?;
        let header = bdd.get_header(hid)?;
        Some(header.label().to_string())
    }

    pub fn get_child(&self, i: usize) -> Option<BddNode> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node)?;
        match node {
            bdd::Node::Zero => None,
            bdd::Node::One => None,
            bdd::Node::Undet => None,
            bdd::Node::NonTerminal(fnode) => Some(BddNode::new(&bddmgr, fnode[i])),
        }
    }

    pub fn is_zero(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node).unwrap();
        match node {
            bdd::Node::Zero => true,
            _ => false,
        }
    }

    pub fn is_one(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node).unwrap();
        match node {
            bdd::Node::One => true,
            _ => false,
        }
    }

    pub fn is_undet(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(self.node).unwrap();
        match node {
            bdd::Node::Undet => true,
            _ => false,
        }
    }

    pub fn dot(&self) -> String {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow().dot_string(self.node);
        result
    }

    pub fn and(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().and(self.node, other.node);
        BddNode::new(&bdd, result)
    }

    pub fn or(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().or(self.node, other.node);
        BddNode::new(&bdd, result)
    }

    pub fn xor(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().xor(self.node, other.node);
        BddNode::new(&bdd, result)
    }

    pub fn not(&self) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().not(self.node);
        BddNode::new(&bdd, result)
    }

    pub fn ite(&self, then: &BddNode, else_: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().ite(self.node, then.node, else_.node);
        BddNode::new(&bdd, result)
    }

    pub fn eq(&self, other: &BddNode) -> bool {
        self.node == other.node
    }

    pub fn prob<T>(&self, pv: HashMap<String, T>, ss: &[bool]) -> T
    where
        T: ProbValue,
    {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = common::HashMap::default();
        let ss = ss.iter().map(|&x| x).collect::<HashSet<bool>>();
        bss_algo::prob(
            &mut bdd.clone().borrow_mut(),
            self.node,
            &pv,
            &ss,
            &mut cache,
        )
    }

    pub fn bmeas<T>(&self, pv: HashMap<String, T>, ss: &[bool]) -> HashMap<String, T>
    where
        T: ProbValue,
    {
        let bdd = self.parent.upgrade().unwrap();
        let ss = ss.iter().map(|&x| x).collect::<HashSet<bool>>();
        bss_algo::bmeas(&mut bdd.clone().borrow_mut(), &ss, self.node, &pv)
    }

    // obtain minimal path vectors (mpvs) of monotone BDD
    pub fn minpath(&self) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache1 = common::HashMap::default();
        let mut cache2 = common::HashMap::default();
        let result = bss_algo::minsol(&mut bdd.borrow_mut(), self.node, &mut cache1, &mut cache2);
        BddNode::new(&bdd, result)
    }

    pub fn bdd_count(&self, ss: &[bool]) -> u64 {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = common::HashMap::default();
        let ss = ss.iter().map(|&x| x).collect::<HashSet<bool>>();
        bss_algo::bdd_count(&mut bdd.clone().borrow_mut(), &ss, self.node, &mut cache)
    }

    pub fn bdd_extract(&self, ss: &[bool]) -> BddPath {
        BddPath::new(self.clone(), ss)
    }

    pub fn zdd_count(&self, ss: &[bool]) -> u64 {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = common::HashMap::default();
        let ss = ss.iter().map(|&x| x).collect::<HashSet<bool>>();
        bss_algo::zdd_count(&mut bdd.clone().borrow_mut(), &ss, self.node, &mut cache)
    }

    pub fn zdd_extract(&self, ss: &[bool]) -> ZddPath {
        ZddPath::new(self.clone(), ss)
    }
}

// enum StackValue {
//     Node(NodeId),
//     Push(String),
//     Pop,
// }

// pub struct BddPath {
//     next_stack: Vec<StackValue>,
//     path: Vec<String>,
//     node: BddNode,
// }

// impl BddPath {
//     pub fn new(node: BddNode) -> Self {
//         let mut next_stack = Vec::new();
//         next_stack.push(StackValue::Node(node.get_id()));
//         BddPath {
//             next_stack: next_stack,
//             path: Vec::new(),
//             node: node,
//         }
//     }

//     pub fn len(&self) -> u64 {
//         self.node.zdd_count(&vec![true])
//     }
// }

// impl Iterator for BddPath {
//     type Item = Vec<String>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let dd = self.node.parent.upgrade().unwrap();
//         while let Some(stackvalue) = self.next_stack.pop() {
//             match stackvalue {
//                 StackValue::Node(node) => match dd.borrow().get_node(node).unwrap() {
//                     bdd::Node::Zero => (),
//                     bdd::Node::One => {
//                         let mut result = self.path.clone();
//                         result.reverse();
//                         return Some(result);
//                     }
//                     bdd::Node::NonTerminal(fnode) => {
//                         let x = dd.borrow().label(node).unwrap().to_string();
//                         self.next_stack.push(StackValue::Pop);
//                         self.next_stack.push(StackValue::Node(fnode[1]));
//                         self.next_stack.push(StackValue::Push(x));
//                         self.next_stack.push(StackValue::Node(fnode[0]));
//                     }
//                     bdd::Node::Undet => (),
//                 },
//                 StackValue::Push(x) => self.path.push(x),
//                 StackValue::Pop => {
//                     self.path.pop();
//                 }
//             }
//         }
//         None
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bss_mgr() {
        let mut bss = BssMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = x.and(&y).or(&z);
        let g = x.and(&y).xor(&z);
        let h = x.and(&y).ite(&z, &x);
        let i = x.and(&y).ite(&z, &y);
        let j = x.and(&y).ite(&z, &x.and(&y));
        let k = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x));
        let l = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y)));
        let m = x
            .and(&y)
            .ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x)));
        let n = x
            .and(&y)
            .ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y))));
    }

    #[test]
    fn test_bss_mgr_prob() {
        let mut bss = BssMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = x.and(&y).or(&z);
        let mut pv = HashMap::new();
        pv.insert("x".to_string(), 0.2);
        pv.insert("y".to_string(), 0.3);
        pv.insert("z".to_string(), 0.6);
        let result = f.prob(pv, &[true]);
        println!("{:?}", result);
    }

    #[test]
    fn test_bss_mgr_rpn() {
        let mut bss = BssMgr::new();
        let x = bss.rpn("x").unwrap();
        let y = bss.rpn("y").unwrap();
        let z = bss.rpn("z").unwrap();
        let f = bss.rpn("x y & z |").unwrap();
    }

    #[test]
    fn test_bdd_path() {
        let mut bss = BssMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.bdd_extract(&[true]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_bdd_path2() {
        let mut bss = BssMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.bdd_extract(&[false]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_zdd_path() {
        let mut bss = BssMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.zdd_extract(&[true]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }
}

