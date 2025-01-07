//

use dd::bdd;
use dd::common::NodeId;
use dd::dot::Dot;
use dd::nodes::DDForest;
use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

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

    pub fn rpn(&mut self, expr: &str) -> Option<BddNode> {
        let mut stack = Vec::new();
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
                _ => {
                    let node = self.defvar(token);
                    stack.push(node.node);
                }
            }
        }
        if let Some(node) = stack.pop() {
            return Some(BddNode::new(&self.bdd, node));
        } else {
            return None;
        }
    }
}

impl BddNode {
    fn getid(&self) -> NodeId {
        self.node
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

    pub fn prob<T>(&self, pv: HashMap<String, T>) -> T
    where
        T: ProbValue,
    {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = HashMap::default();
        bss_algo::prob(&mut bdd.clone().borrow_mut(), self.node, &pv, &mut cache)
    }

    pub fn bmeas<T>(&self, pv: HashMap<String, T>) -> HashMap<String, T>
    where
        T: ProbValue,
    {
        let bdd = self.parent.upgrade().unwrap();
        bss_algo::bmeas(&mut bdd.clone().borrow_mut(), self.node, &pv)
    }

    // obtain minimal path vectors (mpvs) of monotone BDD
    pub fn mpvs(&self) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache1 = HashMap::default();
        let mut cache2 = HashMap::default();
        let result = bss_algo::minsol(
            &mut bdd.clone().borrow_mut(),
            self.node,
            &mut cache1,
            &mut cache2,
        );
        BddNode::new(&bdd, result)
    }

    pub fn extract(&self) -> BddPath {
        BddPath::new(self.clone())
    }

    pub fn size(&self) -> (usize, usize, usize) {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow().size();
        result
    }

    pub fn count_set(&self) -> u64 {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = HashMap::default();
        bss_algo::count_set(&mut bdd.clone().borrow_mut(), self.node, &mut cache)
    }
}

enum StackValue {
    Node(NodeId),
    Push(String),
    Pop,
}

pub struct BddPath {
    next_stack: Vec<StackValue>,
    path: Vec<String>,
    node: BddNode,
}

impl BddPath {
    pub fn new(node: BddNode) -> Self {
        let mut next_stack = Vec::new();
        next_stack.push(StackValue::Node(node.getid()));
        BddPath {
            next_stack: next_stack,
            path: Vec::new(),
            node: node,
        }
    }

    pub fn len(&self) -> u64 {
        self.node.count_set()
    }
}

impl Iterator for BddPath {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let dd = self.node.parent.upgrade().unwrap();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                StackValue::Node(node) => {
                    match dd.borrow().get_node(node).unwrap() {
                        bdd::Node::Zero => (),
                        bdd::Node::One => {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                        bdd::Node::NonTerminal(fnode) => {
                            let x = dd.borrow().label(node).unwrap().to_string();
                            self.next_stack.push(StackValue::Pop);
                            self.next_stack.push(StackValue::Node(fnode[1]));
                            self.next_stack.push(StackValue::Push(x));
                            self.next_stack.push(StackValue::Node(fnode[0]));
                        }
                        bdd::Node::Undet => (),
                    }
                }
                StackValue::Push(x) => self.path.push(x),
                StackValue::Pop => {
                    self.path.pop();
                }
            }
        }
        None
    }
}

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
        let m = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x)));
        let n = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y))));
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
        let path = z.extract();
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }
}
