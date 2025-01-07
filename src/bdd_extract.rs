use dd::bdd::{BddManager, Node};
use dd::common::NodeId;
use dd::nodes::DDForest;


enum StackValue {
    Node(NodeId),
    Push(String),
    Pop,
}

struct BddPath<'a> {
    dd: &'a BddManager,
    next_stack: Vec<StackValue>,
    path: Vec<String>,
    node: NodeId,
}

impl<'a> BddPath<'a> {
    pub fn new(dd: &'a BddManager, node: NodeId) -> Self {
        let mut next_stack = Vec::new();
        next_stack.push(StackValue::Node(node));
        BddPath {
            dd: dd,
            next_stack: next_stack,
            path: Vec::new(),
            node: node,
        }
    }
}

impl<'a> Iterator for BddPath<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                StackValue::Node(node) => {
                    match self.dd.get_node(node).unwrap() {
                        Node::Zero => (),
                        Node::One => {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                        Node::NonTerminal(fnode) => {
                            let x = self.dd.label(node).unwrap();
                            self.next_stack.push(StackValue::Pop);
                            self.next_stack.push(StackValue::Node(fnode[1]));
                            self.next_stack.push(StackValue::Push(x.to_string()));
                            self.next_stack.push(StackValue::Node(fnode[0]));
                        }
                        Node::Undet => (),
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

pub fn bdd_path(dd: &BddManager, node: NodeId) -> BddPath {
    BddPath::new(dd, node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dd::bdd::BddManager;
    use dd::bdd::Node;
    use dd::common::NodeId;
    use dd::dot::Dot;
    use dd::nodes::DDForest;

    #[test]
    fn test_bdd_path() {
        let mut bdd = BddManager::new();
        let hx = bdd.create_header(0, "x");
        let hy = bdd.create_header(1, "y");
        let hz = bdd.create_header(2, "z");
        let x = bdd.create_node(hx, bdd.zero(), bdd.one());
        let y = bdd.create_node(hy, bdd.zero(), bdd.one());
        let z = bdd.create_node(hz, bdd.zero(), bdd.one());
        let tmp = bdd.or(y, z);
        let f = bdd.and(x, tmp);
        println!("{}", bdd.dot_string(f));
        let path = bdd_path(&bdd, f);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_bdd_path2() {
        let mut bdd = BddManager::new();
        let hx = bdd.create_header(0, "x");
        let hy = bdd.create_header(1, "y");
        let hz = bdd.create_header(2, "z");
        let x = bdd.create_node(hx, bdd.zero(), bdd.one());
        let y = bdd.create_node(hy, bdd.zero(), bdd.one());
        let z = bdd.create_node(hz, bdd.zero(), bdd.one());
        let tmp = bdd.or(y, z);
        let f = bdd.or(x, tmp);
        println!("{}", bdd.dot_string(f));
        let path = bdd_path(&bdd, f);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }
}

