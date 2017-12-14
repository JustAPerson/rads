//! Maximum number of children below a node
use std::rc::Rc;
use std::cell::RefCell;

pub const B: usize = 7;
/* const */ fn midpoint() -> usize {
    debug_assert!(B % 2 == 1, "B must be odd");
    // cut in half rounding down.
    // This provides a zero-based index into the middle of an array of length B
    // E.g. if B=7, then B/2 = 3 which is middle index of an array of 7
    B / 2
}

pub type BTreePtr<K, V> = Rc<RefCell<BTreeNode<K, V>>>;
fn make_ptr<K: Ord, V>(node: BTreeNode<K, V>) -> BTreePtr<K, V> {
    Rc::new(RefCell::new(node))
}

pub struct BTree<K, V> where K: Ord {
    root: BTreePtr<K, V>,
    size: usize,
}

pub struct BTreeNode<K, V> where K: Ord {
    parent: Option<BTreePtr<K, V>>,
    children: Vec<BTreePtr<K, V>>,
    items: Vec<(K, V)>,
}

impl<K: Ord, V> BTree<K, V> {
    pub fn new() -> Self {
        BTree {
            root: make_ptr(BTreeNode::new_root()),
            size: 0,
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        let node = self.find_insert(k);
        node.
    }

    fn find_insert(&self, k: K) -> &BTreeNode<K, V> {
        if self.root.borrow().items.len() == 0 {
            self.root
        } else {
        }
    }

    fn print(&self) {
        // self.root.borrow().print(0);
    }
}

impl<K: Ord, V> BTreeNode<K, V> {
    fn new_root() -> Self {
        BTreeNode {
            parent: None,
            children: Vec::with_capacity(B),
            items: Vec::with_capacity(B),
        }
    }

    fn new(parent: &BTreePtr<K, V>) -> Self {
        BTreeNode {
            parent: Some(parent.clone() /* clone RC ptr */),
            children: Vec::with_capacity(B),
            items: Vec::with_capacity(B),
        }
    }

    fn open(&self) -> bool {
        self.items.len() <= B
    }

    fn leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn split(&mut self) -> ((K, V), BTreePtr<K, V>){
        debug_assert!(self.items.len() == B);
        debug_assert!(self.leaf() || self.children.len() == B + 1);

        // left: leave 4 items and 4 children
        // right: take 3 items and 4 children
        let right_items = self.items.split_off(midpoint() + 1);
        let right_children = if self.leaf() {
            Vec::new()
        } else {
            self.children.split_off(midpoint() + 1)
        };

        // Take one item from left to become midpoint
        let midpoint = self.items.pop().unwrap();

        let right = make_ptr(BTreeNode {
            parent: self.parent.clone() /* clone Option<Rc<_>> */,
            items: right_items,
            children: right_children,
        });

        (midpoint, right)
    }

    fn upinsert(&mut self, kv: (K, V), right: BTreePtr<K, V>) {
        debug_assert!(!self.leaf());

        let index = match self.items.binary_search_by(|&(ref l, _)| l.cmp(&kv.0)) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.items.insert(index, kv);
        self.children.insert(index, right);
    }

    fn balance(this: &BTreePtr<K, V>) {
        if this.borrow().items.len() == B {
            let (midpoint, right) = this.borrow_mut().split();

            if this.borrow().parent.is_some() {
                let this = this.borrow();
                let parent = this.parent.as_ref().unwrap();
                right.borrow_mut().parent = Some(parent.clone());
                parent.borrow_mut().upinsert(midpoint, right);
                BTreeNode::balance(parent);
            } else {
                // create new root
                let mut root = BTreeNode::new_root();
                root.items.push(midpoint);
                root.children.push(this.clone());
                root.children.push(right);

                // reassign parents of left/right
                let root = make_ptr(root);
                this.borrow_mut().parent = Some(root.clone());
                this.borrow_mut().parent = Some(root);
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_btree() {
        let mut b: BTree<usize, usize> = BTree::new();
        b.insert(3, 5);
        assert!(b.get)
    }
}
