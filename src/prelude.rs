
pub use std::collections::{HashMap, HashSet, VecDeque};
pub use std::fmt::Debug;

pub use std::ops::{Add, Mul, Sub, Div};
use std::str::FromStr;

pub use dd::common::HeaderId;
pub use dd::common::Level;
pub use dd::common::NodeId;
pub use dd::dot::Dot;
pub use dd::nodes::DDForest;
pub use dd::nodes::NonTerminal;
pub use dd::nodes::Terminal;

pub use dd::bdd;
pub use dd::bdd::BddManager;

pub use dd::mdd;
pub use dd::mtmdd;
pub use dd::mtmdd2;
pub use dd::mtmdd2::MtMdd2Manager;

pub type BddHashMap<K,V> = dd::common::HashMap<K,V>;

pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use std::rc::Weak;

pub trait ProbValue:
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Clone + Copy + PartialEq + From<f64>
{
}

impl ProbValue for f64 {}

pub trait MDDValue: dd::common::TerminalNumberValue + From<i32> + FromStr {}

impl MDDValue for i64 {}
impl MDDValue for i32 {}