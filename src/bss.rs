//

use dd::bdd;
use dd::common::NodeId;
use dd::dot::Dot;
use std::collections::HashSet;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use crate::bss_algo;
use crate::bss_algo::ProbValue;

pub struct BssMgr {
    bdd: Rc<RefCell<bdd::BddManager>>,
    vars: HashMap<String, NodeId>,
}

#[derive(Clone)]
pub struct Node {
    parent: Weak<RefCell<bdd::BddManager>>,
    node: NodeId,
}

impl Node {
    pub fn new(bdd: &Rc<RefCell<bdd::BddManager>>, node: NodeId) -> Self {
        Node {
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
            vars: HashMap::new(),
        }
    }

    // size
    pub fn size(&self) -> (usize, usize, usize) {
        self.bdd.borrow().size()
    }

    // zero
    pub fn zero(&self) -> Node {
        Node::new(&self.bdd, self.bdd.borrow().zero())
    }

    // one
    pub fn one(&self) -> Node {
        Node::new(&self.bdd, self.bdd.borrow().one())
    }

    // defvar
    pub fn defvar(&mut self, var: &str) -> Node {
        if let Some(node) = self.vars.get(var) {
            return Node::new(&self.bdd, *node);
        } else {
            let level = self.vars.len();
            let mut bdd = self.bdd.borrow_mut();
            let h = bdd.create_header(level, var);
            let x0 = bdd.zero();
            let x1 = bdd.one();
            let node = bdd.create_node(h, x0, x1);
            self.vars.insert(var.to_string(), node);
            Node::new(&self.bdd, node)
        }
    }

    pub fn rpn(&mut self, expr: &str) -> Option<Node> {
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
            return Some(Node::new(&self.bdd, node));
        } else {
            return None;
        }
    }
}

impl Node {
    pub fn dot(&self) -> String {
            let bdd = self.parent.upgrade().unwrap();
            let result = bdd.borrow().dot_string(self.node);
            result
    }

    pub fn and(&self, other: &Node) -> Node {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().and(self.node, other.node);
        Node::new(&bdd, result)
    }

    pub fn or(&self, other: &Node) -> Node {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().or(self.node, other.node);
        Node::new(&bdd, result)
    }

    pub fn xor(&self, other: &Node) -> Node {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().xor(self.node, other.node);
        Node::new(&bdd, result)
    }

    pub fn not(&self) -> Node {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().not(self.node);
        Node::new(&bdd, result)
    }

    pub fn ite(&self, then: &Node, else_: &Node) -> Node {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().ite(self.node, then.node, else_.node);
        Node::new(&bdd, result)
    }

    pub fn eq(&self, other: &Node) -> bool {
        self.node == other.node
    }

    pub fn prob<T>(&self, pv: HashMap<String, T>) -> T
    where
        T: ProbValue,
    {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = HashMap::new();
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
        let mut cache1 = HashMap::new();
        let mut cache2 = HashMap::new();
        let result = bdd_algo::minsol(
            &mut bdd.clone().borrow_mut(),
            &self.node,
            &mut cache1,
            &mut cache2,
        );
        BddNode::new(bdd.clone(), result)
    }

    pub fn extract(&self) -> Vec<Vec<String>> {
        let mut path = Vec::new();
        let mut pathset = Vec::new();
        bdd_algo::extract(&self.node, &mut path, &mut pathset);
        pathset
    }

    pub fn size(&self) -> (usize, u64) {
        self.node.count()
    }

    pub fn count_set(&self) -> u64 {
        let mut cache = HashMap::new();
        bdd_algo::count_set(&self.node, &mut cache)
    }
}
