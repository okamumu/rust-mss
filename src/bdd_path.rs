use std::collections::HashSet;

use dd::bdd;
use dd::common::NodeId;
use dd::nodes::DDForest;

use crate::bss::BddNode;

enum StackValue {
    Node(NodeId),
    Push(String),
    Pop,
}

pub struct ZddPath {
    next_stack: Vec<StackValue>,
    path: Vec<String>,
    node: BddNode,
    ss: HashSet<bool>,
}

impl ZddPath {
    pub fn new(node: BddNode, ss: &[bool]) -> Self {
        let mut next_stack = Vec::new();
        next_stack.push(StackValue::Node(node.get_id()));
        let ss = ss.iter().cloned().collect::<HashSet<_>>();
        ZddPath {
            next_stack: next_stack,
            path: Vec::new(),
            node: node,
            ss: ss,
        }
    }

    pub fn len(&self) -> u64 {
        self.node.zdd_count(&vec![true])
    }
}

impl Iterator for ZddPath {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        // let dd = self.node.parent.upgrade().unwrap();
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                StackValue::Node(node) => match dd.borrow().get_node(node).unwrap() {
                    bdd::Node::Zero => {
                        if self.ss.contains(&false) {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                    }
                    bdd::Node::One => {
                        if self.ss.contains(&true) {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                    }
                    bdd::Node::NonTerminal(fnode) => {
                        let x = dd.borrow().label(node).unwrap().to_string();
                        self.next_stack.push(StackValue::Pop);
                        self.next_stack.push(StackValue::Node(fnode[1]));
                        self.next_stack.push(StackValue::Push(x));
                        self.next_stack.push(StackValue::Node(fnode[0]));
                    }
                    bdd::Node::Undet => (),
                },
                StackValue::Push(x) => self.path.push(x),
                StackValue::Pop => {
                    self.path.pop();
                }
            }
        }
        None
    }
}
