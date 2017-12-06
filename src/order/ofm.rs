//! Order File Maintenance
//!
//! This structure is similar to a vector supporting fast insertion. In a
//! traditional vector, insertion/deletion requires `O(n)` time to shift all the
//! succeeding elements around. By carefully maintaining constant-sized gaps
//! between elements, we can speed up insertion/deletion to requiring `O(log^2 n)`
//! time.
//!
//! This could better be accomplished by a linked list. However, this structure
//! is cache-oblivious, meaning it performs an asymptotically optimal number of
//! memory transfers between all cache hierarchy levels. In effect, in order
//! traversal becomes much faster due to fewer cache-misses.

use std;
use std::ops::Range;

/// Construct an array of Nones
///
/// Useful if T: !Clone
fn empty_array<T>(n: usize) -> Box<[Option<T>]> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(None)
    }
    v.into_boxed_slice()
}

/// Determine if a range is ripe for rebalancing
fn threshold(d: f32, rho: f32) -> bool {
    (rho >= 0.5 - d/4.0) && (rho <= 0.75 + d/4.0)
}

/// An opaque wrapper
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Index(usize);

// Default impl does nothing, can be specialized for more interesting semantics
pub trait Indexable {
    fn index(&mut self, new: Index);
}
default impl<T> Indexable for T {
    fn index(&mut self, _: Index) {}
}

/// Order File Maintenance
///
/// Faster insertion than an array; faster traversal than a
/// linked list.
#[derive(Clone, Debug)]
pub struct Ofm<T: Indexable> {
    cells: Box<[Option<T>]>,
    occupied: Box<[usize]>,
    leaf_size: usize,
    size: usize,
}

impl<T: Indexable> Ofm<T> {
    /// Construct a new instance
    ///
    /// This will allocate a small amount of space, unlike `Vec::new()`.
    pub fn new() -> Self {
        Ofm {
            cells: empty_array(2),
            occupied: vec![0; 2].into_boxed_slice(),
            leaf_size: 1,
            size: 0,
        }
    }

    pub fn push_front(&mut self, v: T) {
        self.insert(0, v);
    }

    pub fn push_back(&mut self, v: T) {
        let n = self.cells.len();
        self.insert(n, v)
    }

    fn insert(&mut self, i: usize, v: T) {
        self.size += 1;

        let (mut leaf, mut offset) = self.leaf(i);
        if i == self.cells.len() {
            leaf -= 1;
            offset = self.leaf_size - 1;
        }

        self.grow(leaf);
        let r = self.leaf_boundary(leaf);

        let mut vals = self.cells_take(r.clone());
        let num_vals = vals.len();
        debug_assert!(num_vals != self.leaf_size);

        vals.insert(offset.min(num_vals), v);
        self.redistribute(r, vals);
    }

    fn grow(&mut self, l: usize) {
        if self.occupied[l] == self.leaf_size {
            let leaves = self.occupied.len();
            let height = (2f32 * leaves as f32).log2() as u32;
            debug_assert!(2usize.pow(height) == 2*leaves); // perfect power of 2 => complete binary tree

            let tree = conceptual_tree::Tree::new(height);
            let mut node = tree.get_leaf(l);

            let mut o = self.occupied[l];
            let mut c = self.leaf_size;
            loop {
                let r = tree.range(node.sibling());
                c *= 2;
                o += self.occupied[r].iter().sum::<usize>();
                node.parent();

                if node.is_root() {
                    self.double();
                    return;
                } else if threshold(node.depth as f32 / (height - 0) as f32, o as f32 / c as f32) {
                    node.parent();
                    let mut r = tree.range(node);
                    r.start *= self.leaf_size;
                    r.end   *= self.leaf_size;
                    self.rebalance(r);
                    return;
                }
            }
        }
    }

    fn double(&mut self) {
        use std::mem;

        // TODO doubling strategy?
        self.leaf_size += 1;
        let num_leaves = self.occupied.len() * 2;
        let num_cells = self.leaf_size * num_leaves;

        self.occupied = vec![0; num_leaves].into_boxed_slice();
        let cells: Vec<_> = mem::replace(&mut self.cells, empty_array(num_cells)).into();

        self.redistribute(0..num_cells, cells.into_iter().filter_map(|c| c).collect());
    }

    /// Returns (index, offset)
    fn leaf(&self, i: usize) -> (usize, usize) {
        let divisor = 1.max(self.leaf_size); // don't divide by zero
        (i / divisor, i % divisor)
    }

    fn leaf_boundary(&self, l: usize) -> Range<usize> {
        let l_start = (l + 0) * self.leaf_size;
        let l_end   = (l + 1) * self.leaf_size;
        l_start .. l_end
    }

    fn cell_take(&mut self, i: usize) -> Option<T> {
        let val = self.cells[i].take();
        if val.is_some() {
            let leaf = self.leaf(i);
            self.occupied[leaf.0] -= 1;
        }
        val
    }

    fn cell_put(&mut self, i: usize, mut v: T) {
        v.index(Index(i));
        self.cells[i] = Some(v);

        let leaf = self.leaf(i);
        self.occupied[leaf.0] += 1;
    }

    fn cells_take(&mut self, r: Range<usize>) -> Vec<T> {
        r.into_iter().filter_map(|i| self.cell_take(i)).collect()
    }

    fn rebalance(&mut self, r: Range<usize>) {
        let vs = self.cells_take(r.clone());
        self.redistribute(r, vs);
    }

    fn redistribute(&mut self, r: Range<usize>, vs: Vec<T>) {
        let stride = r.len() / vs.len();
        let mut i = r.start;
        for v in vs {
            self.cell_put(i, v);
            i += stride;
        }
    }
}

impl<T> std::ops::Index<Index> for Ofm<T> {
    type Output = T;
    fn index(&self, i: Index) -> &T {
        self.cells[i.0 as usize].as_ref().expect("Invalid index")
    }
}

pub struct OfmIter<'a, T: 'a> {
    cells: &'a [Option<T>],
    i: usize,
}

impl<'a, T> std::iter::Iterator for OfmIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.cells.len() {
            let data = self.cells[self.i].as_ref();
            self.i += 1;
            if data.is_some() {
                return data
            }
        }
        None
    }
}
// TODO DoubleSidedIterator, ExactSizeIterator, size_hint

impl<'a, T: 'a> std::iter::IntoIterator for &'a Ofm<T> {
    type Item = &'a T;
    type IntoIter = OfmIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        OfmIter { cells: &self.cells, i: 0 }
    }
}


mod conceptual_tree {
    use std::ops::Range;

    pub struct Tree {
        pub height: u32,
    }
    #[derive(Copy, Clone)]
    pub struct Node {
        pub depth: u32,
        pub offset: usize,
    }

    impl Tree {
        pub fn new(height: u32) -> Tree {
            Tree { height }
        }

        /// Number of leaves
        fn size(&self) -> usize { 2usize.pow(self.height - 1) }

        pub fn get_leaf(&self, index: usize) -> Node {
            debug_assert!(index < self.size());
            Node {
                depth: self.height - 1,
                offset: index,
            }
        }

        pub fn range(&self, node: Node) -> Range<usize> {
            debug_assert!(node.depth < self.height);
            let width = 2usize.pow(self.height - node.depth - 1);
            let offset = node.offset * width;
            (offset .. offset + width)
        }
    }

    impl Node {
        pub fn is_root(&self) -> bool {
            self.depth == 0
        }

        /// Transform node to its parent
        pub fn parent(&mut self) {
            debug_assert!(!self.is_root());
            self.depth -= 1;
            self.offset /= 2;
        }

        pub fn sibling(&self) -> Node {
            Node {
                depth: self.depth,
                offset: self.offset ^ 0b1 // flip parity
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        /// Generate sequence of ranges walking leaf-to-root path
        fn ranges(t: &Tree, index: usize) -> Vec<Range<usize>> {
            let mut output = Vec::new();
            let mut n = t.get_leaf(index);
            output.push(t.range(n));
            for _ in 0..(t.height - 1) {
                n.parent();
                output.push(t.range(n));
            }
            output
        }

        #[test]
        fn range() {
            let t = Tree::new(5);
            assert_eq!(ranges(&t, 0),  [ 0..1 ,  0..2 ,  0..4 , 0..8 , 0..16]);
            assert_eq!(ranges(&t, 15), [15..16, 14..16, 12..16, 8..16, 0..16]);
            assert_eq!(ranges(&t, 3),  [ 3..4 ,  2..4 ,  0..4 , 0..8 , 0..16]);
            assert_eq!(ranges(&t, 8),  [ 8..9 ,  8..10,  8..12, 8..16, 0..16]);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::Bencher;
    use order::test::N;

    #[test]
    fn test_ofm_back() {
        let mut o = Ofm::<usize>::new();
        o.push_back(1);
        o.push_back(2);
        o.push_back(3);
        o.push_back(4);
        assert_eq!(o.into_iter().cloned().collect::<Vec<usize>>(), [1usize, 2, 3, 4]);
    }

    #[test]
    fn test_ofm_front() {
        let mut o = Ofm::<usize>::new();
        o.push_front(1);
        o.push_front(2);
        o.push_front(3);
        o.push_front(4);
        assert_eq!(o.into_iter().cloned().collect::<Vec<usize>>(), [4usize, 3, 2, 1]);
    }

    #[test]
    fn test_ofm_moving() {
        struct Atom(usize);
        impl Indexable for Atom {
            fn index(&mut self, _: Index) {
                self.0 += 1;
            }
        }

        let mut o = Ofm::<Atom>::new();
        o.push_back(Atom(0));
        o.push_back(Atom(0));
        o.push_back(Atom(0));
        o.push_back(Atom(0));
        assert!(o.into_iter().map(|a| a.0).all(|n| n > 0))
    }

    #[bench]
    fn bench_ofm_push_back(b: &mut Bencher) {
        b.iter(|| {
            let mut o = Ofm::new();
            for i in 0..N { o.push_back(i) }
        })
    }

    #[bench]
    fn bench_ofm_push_front(b: &mut Bencher) {
        b.iter(|| {
            let mut o = Ofm::new();
            for i in 0..N { o.push_front(i) }
        })
    }

    #[bench]
    fn bench_ofm_iter(b: &mut Bencher) {
        let mut o = Ofm::new();
        for i in 0..N { o.push_back(i) }
        b.iter(|| o.into_iter().sum::<usize>());
    }
}
