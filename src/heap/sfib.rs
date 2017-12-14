//! Strict Fibonacci Heaps
//!
//! TODO enforce unique keys assumption

use std::rc::Rc;
use std::cell::{Cell, RefCell, Ref};
use std::collections::VecDeque;

use util::CyclicList;

enum RankDesc<K, V> {
    Rank(usize),
    Fix(FixPtr<K, V>),
    None,
}

impl<K, V> RankDesc<K, V> {
    // fn increase(&mut self) {
    //     match *self {
    //         RankDesc::None => self = RankDesc::Rank(0),
    //         RankDesc::Rank(usize)
    //     }
    // }
    fn increase(&mut self) {
        unimplemented!()
    }
    fn decrease(&mut self) {
        unimplemented!()
    }
    
}

type FixPtr<K, V> = CyclicList<Fix<K, V>>;
struct Fix<K, V> {
    node: NodePtr<K, V>,
    rank: usize,
}

type NodePtr<K, V> = CyclicList<RefCell<Node<K, V>>>;
struct Node<K, V> {
    key: K,
    val: V,
    active: Option<Rc<Cell<bool>>>,
    rank: RankDesc<K, V>,
    loss: usize, // potential

    parent: Option<NodePtr<K, V>>,
    children: VecDeque<NodePtr<K, V>>,
}

impl<K: Ord, V> Node<K, V> {
    fn new(key: K, val: V) -> Self {
        Node {
            key,
            val,
            active: None,
            rank: RankDesc::None,
            loss: 0,

            parent: None,
            children: VecDeque::new(),
        }
    }

    fn new_ptr(key: K, val: V) -> NodePtr<K, V> {
        NodePtr::new(RefCell::new(Self::new(key, val)))
    }
}

impl<K, V> Node<K, V> {
    fn is_active(&self) -> bool { self.active.as_ref().map_or(false, |b| b.get()) }
    fn is_passive(&self) -> bool { ! self.is_active() }

    fn is_active_root(&self) -> bool {
        let parent = match self.parent {
            Some(ref parent) => parent.borrow().active.is_none(),
            None => true,
        };

        self.is_active() && parent
    }

    fn is_linkable(&self) -> bool {
        self.children.iter().all(|c| c.borrow().is_passive())
    }

    fn is_passive_linkable(&self) -> bool {
        self.is_passive() && self.is_linkable()
    }
}

fn child_index<K, V>(node: &NodePtr<K, V>, child: &NodePtr<K, V>) -> usize {
    node.borrow().children.iter().enumerate()
        .filter(|&(_, c)| NodePtr::ptr_eq(c, child))
        .next().unwrap().0
}

fn child_remove<K, V>(node: &NodePtr<K, V>, child: &NodePtr<K, V>) {
    let i = child_index(node, child);
    node.borrow_mut().children.remove(i);
}

pub struct Sfib<K, V> {
    size: usize,
    root: Option<NodePtr<K, V>>,
    active: Rc<Cell<bool>>,

    // correspond to the 4 parts of the fix-list
    q: Option<NodePtr<K, V>>,
    fix_multis: Option<FixPtr<K, V>>,
    fix_singles: Option<FixPtr<K, V>>,
}

pub struct Element<K, V>(NodePtr<K, V>);

impl<K: Ord, V> Sfib<K, V> {
    pub fn new() -> Self {
        Sfib {
            size: 0,
            root: None,
            active: Rc::new(Cell::new(true)),

            q: None,
            fix_multis: None,
            fix_singles: None,
        }

    }

    pub fn min_key(&self) -> Option<Ref<K>> {
        self.root.as_ref().map(|r| Ref::map(r.borrow(), |n| &n.key))
    }

    pub fn min_val(&self) -> Option<Ref<V>> {
        self.root.as_ref().map(|r| Ref::map(r.borrow(), |n| &n.val))
    }

    pub fn min_node(&self) -> Option<Element<K, V>> {
        self.root.as_ref().map(|r| Element(r.clone()))
    }

    pub fn insert(&mut self, key: K, val: V) -> Element<K, V> {
        if self.root.is_none() {
            let root = Node::new_ptr(key, val);
            self.root = Some(root.clone());
            self.size = 1;
            Element(root)
        } else {
            let mut other = Self::new();
            let elem = other.insert(key, val);
            self.meld(other);
            elem
        }
    }

    pub fn meld(&mut self, mut other: Self) {
        if other.root.is_none() { return }
        if self.root.is_none() { *self = other; return }

        debug_assert!(self.root.as_ref().unwrap().borrow().is_passive());
        debug_assert!(other.root.as_ref().unwrap().borrow().is_passive());

        // make all nodes of smaller heap passive
        if self.size <= other.size {
            self.active.set(false);
        } else {
            other.active.set(false);
        }

        self.size += other.size; // combine sizes

        // rename u/v such that u < v
        let (u, v) = (self.root.take().unwrap(), other.root.take().unwrap());
        let (u, v) = if u.borrow().key < v.borrow().key { (u, v) } else { (v, u) };

        // let u be root, and v its child
        v.borrow_mut().parent = Some(u.clone());
        u.borrow_mut().children.push_back(v.clone());
        self.root = Some(u);

        match (self.q.take(), other.q.take()) {
            (Some(a), Some(b)) => {
                a.push_back(v);
                a.extend_back(b);
                self.q = Some(a);
            }
            (Some(a), None) => {
                a.push_back(v);
                self.q = Some(a)
            }
            (None, Some(b)) => {
                v.extend_back(b);
                self.q = Some(v);
            }
            (None, None) => self.q = Some(v),
        }

        self.reduce(1, 1, 0, 0);
    }

    fn reduce(&mut self, mut a: usize, mut b: usize, mut c: usize, mut d: usize) {
        let mut progress = true;
        let mut sum = a + b + c + d;
        while progress & (sum > 0) {
            if a > 0 { if self.active_root_reduction() { a -= 1 } }
            if b > 0 { if self.root_degree_reduction() { b -= 1 } }
            if c > 0 { if self.one_node_loss_reduction() { c -= 1 } }
            if d > 0 { if self.two_node_loss_reduction() { d -= 1 } }

            let old_sum = sum;
            sum = a + b + c + d;
            progress = sum < old_sum; // progress if at least one thing happened
        }
    }

    fn link(&mut self, x: NodePtr<K, V>, y: &NodePtr<K, V>) {
        let active = x.borrow().is_active();

        if let Some(parent) = x.borrow_mut().parent.take() {
            child_remove(&parent, &x);
            if active { parent.borrow_mut().rank.decrease() }
        }

        x.borrow_mut().parent = Some(y.clone());
        if active {
            let mut y = y.borrow_mut();
            y.children.push_front(x); // TODO eliminate clone?
            y.rank.increase();
        } else {
            y.borrow_mut().children.push_back(x);
        }
    }

    fn reparent(&self, x: NodePtr<K, V>, y: &NodePtr<K, V>) {
        debug_assert!(x.borrow().is_passive());
        let parent = x.borrow_mut().parent.take(); // borrowck: nll cannot come soon enough
        if let Some(parent) = parent {
            child_remove(&parent, &x);
            x.borrow_mut().parent = Some(y.clone());
            y.borrow_mut().children.push_back(x);
        }
    }

    fn activate(&mut self, x: &NodePtr<K, V>) {
        // change parent rank
        // check if active root and fix lis
    }
    fn deactivate(&mut self, x: &NodePtr<K, V>) {}

    fn active_root_reduction(&mut self) -> bool {
        let x = if let Some(ref multis) = self.fix_multis { multis.next().clone() }
                else { return false };
        let y = x.next().clone();
        if FixPtr::ptr_eq(&x, &y) || x.rank != y.rank {
            return false
        }

        let (x, y) = if x.node.borrow().key < y.node.borrow().key { (x, y) } else { (y, x) };
        self.link(y.node.clone(), &x.node);

        let mut borrow = x.node.borrow_mut();
        if borrow.children.back().unwrap().borrow().is_passive() {
            let z = borrow.children.pop_back().unwrap();
            self.reparent(z, self.root.as_ref().unwrap());
        }

        true
    }

    fn root_degree_reduction(&mut self) -> bool{
        true
    }

    fn one_node_loss_reduction(&mut self) -> bool {
        true
    }

    fn two_node_loss_reduction(&mut self) -> bool{
        true
    }
}

