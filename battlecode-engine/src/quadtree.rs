#![allow(dead_code)]

//! A reasonably fast quadtree implementation.
use smallvec::{SmallVec, Array};
use std::fmt::Debug;

/// A coordinate representable in the quadtree.
pub type Coord = u8;
/// A location in the quadtree.
pub type Loc = (Coord, Coord);
/// The maximum possible coordinate.
pub const MIN_COORD: Coord = 0;
/// The maximum possible coordinate.
pub const MAX_COORD: Coord = 255;
/// The size of the world represented by the quadtree.
pub const SIZE: usize = 256;
/// The maximum depth of the quadtree.
pub const DEPTH: u8 = 5;

const SPLIT_THRESH: usize = 16;

/// A "pointer" to a node; actually an index into a backing vector.
type NodePtr = u16;

macro_rules! idx {
    ($arr:expr, $i:expr) => {
        unsafe { $arr.get_unchecked($i) }
    };
}
macro_rules! idx_mut {
    ($arr:expr, $i:expr) => {
        unsafe { $arr.get_unchecked_mut($i) }
    };
}

/// An entry in the quadtree.
#[derive(Debug, Clone)]
struct Entry<T: Clone> {
    loc: Loc,
    value: T
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Corner {
    TL=0, TR=1, BL=2, BR=3
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    minx: Coord,
    maxx: Coord,
    miny: Coord,
    maxy: Coord
}
impl Bounds {
    fn top_level() -> Bounds {
        return Bounds {
            minx: MIN_COORD,
            maxx: MAX_COORD,
            miny: MIN_COORD,
            maxy: MAX_COORD,
        }
    }
    fn check(&self) {
        debug_assert!(self.minx <= self.maxx && self.miny <= self.maxy);
    }
    #[inline(always)]
    fn split(&self) -> [Bounds; 4] {
        let Bounds {minx, maxx, miny, maxy} = *self;
        // don't do (minx + maxx)/2, that can overflow
        let midx = (minx / 2) + (maxx / 2);
        let midy = (miny / 2) + (maxy / 2);
        let children = [
            // TL (x: 0, y: 1)
            Bounds { minx: minx, maxx: midx-1, miny: midy, maxy: maxy },

            // TR (x: 1, y: 1)
            Bounds { minx: midx, maxx: maxx, miny: midy, maxy: maxy },

            // BL (x: 0, y: 0)
            Bounds { minx: minx, maxx: midx-1, miny: miny, maxy: midy-1},

            // BR (x: 1, y: 0)
            Bounds { minx: midx, maxx: maxx, miny: miny, maxy: midy-1}
        ];
        //debug_assert!(self.contains_other(&children[0]));
        //debug_assert!(self.contains_other(&children[1]));
        //debug_assert!(self.contains_other(&children[2]));
        //debug_assert!(self.contains_other(&children[3]));
        //debug_assert!(!children[0].overlaps(&children[1]));
        //debug_assert!(!children[0].overlaps(&children[2]));
        //debug_assert!(!children[0].overlaps(&children[3]));
        //debug_assert!(!children[1].overlaps(&children[2]));
        //debug_assert!(!children[1].overlaps(&children[3]));
        //debug_assert!(!children[2].overlaps(&children[3]));

        children
    }
    #[inline(always)]
    fn contains(&self, loc: Loc) -> bool {
        self.minx <= loc.0 &&
        self.miny <= loc.1 &&
        self.maxx >= loc.0 &&
        self.maxy >= loc.1
    }
    fn contains_other(&self, other: &Bounds) -> bool {
        self.minx <= other.minx &&
        self.miny <= other.miny &&
        self.maxx >= other.maxx &&
        self.maxy >= other.maxy
    }
    #[inline(always)]
    fn overlaps(&self, other: &Bounds) -> bool {
        if self.maxx < other.minx || other.maxx < self.minx {
            false
        } else if self.maxy < other.miny || other.maxy < self.miny {
            false
        } else {
            true
        }
    }
    #[inline(always)]
    fn child(&self, loc: Loc) -> Corner {
        let Bounds {minx, maxx, miny, maxy} = *self;
        let midx = minx / 2 + maxx / 2;
        let midy = miny / 2 + maxy / 2;
        if loc.0 < midx {
            if loc.1 < midy {
                Corner::BL
            } else {
                Corner::TL
            }
        } else {
            if loc.1 < midy {
                Corner::BR
            } else {
                Corner::TR
            }
        }
    }
}

/// A quadtree node.
#[derive(Debug, Clone)]
enum Node<T: Clone> {
    Branch {
        bounds: Bounds,
        children: [NodePtr; 4],
    },
    Leaf {
        bounds: Bounds,
        elements: SmallVec<[Entry<T>; 8]>,
    }
}
impl<T: Clone> Node<T> {
    fn bounds(&self) -> &Bounds {
        match self {
            &Node::Branch { ref bounds, .. } => bounds,
            &Node::Leaf { ref bounds, .. } => bounds,
        }
    }
}

/// A quadtree.
/// The quadtree maps locations - (u8, u8) tuples - to arbitrary values.
/// If your data doesn't fit in the range (0, 255)... make it fit.
/// Multiple keys cannot be stored in the same location.
#[derive(Clone)]
pub struct Quadtree<T: Clone> {
    /// The nodes of the quadtree.
    /// The root is always node 0.
    nodes: Vec<Node<T>>
}

impl <T: Clone + Debug> Quadtree<T> {
    /// Create a new, empty quadtree.
    pub fn new() -> Quadtree<T> {
        Quadtree {
            nodes: vec![
                // 0 is the root node
                Node::Leaf {
                    bounds: Bounds::top_level(),
                    elements: SmallVec::new(),
                }
            ]
        }
    }

    /// Lookup the node for a given location.
    fn lookup(&self, loc: Loc) -> (NodePtr, u8) {
        let mut ptr = 0usize;
        let mut depth = 0u8;

        loop {
            match idx!(self.nodes, ptr) {
                &Node::Branch { ref bounds, ref children, .. } => {
                    ptr = (*idx!(children, bounds.child(loc) as usize)) as usize;
                    depth += 1;
                }
                &Node::Leaf { .. } => break
            }
        }
        (ptr as NodePtr, depth)
    }

    /// Insert a value into the quadtree.
    pub fn insert(&mut self, loc: Loc, value: T) {
        let (ptr, depth) = self.lookup(loc);

        let (bounds, children) =
            if let &mut Node::Leaf { bounds, ref mut elements } = idx_mut!(self.nodes, ptr as usize) {
            if elements.len() < SPLIT_THRESH || depth == DEPTH {
                if cfg!(debug) {
                    for elem in elements.iter() {
                        assert!(elem.loc != loc);
                    }
                }
                elements.push(Entry{ loc, value });
                return;
            } else {
                //println!("SPLIT {:?}", bounds);
                // split node
                let mut child_elements = (
                    SmallVec::new(),
                    SmallVec::new(),
                    SmallVec::new(),
                    SmallVec::new()
                );
                for entry in elements.drain() {
                    match bounds.child(entry.loc) {
                        Corner::TL => child_elements.0.push(entry),
                        Corner::TR => child_elements.1.push(entry),
                        Corner::BL => child_elements.2.push(entry),
                        Corner::BR => child_elements.3.push(entry),
                    }
                }
                match bounds.child(loc) {
                    Corner::TL => child_elements.0.push(Entry{ loc, value }),
                    Corner::TR => child_elements.1.push(Entry{ loc, value }),
                    Corner::BL => child_elements.2.push(Entry{ loc, value }),
                    Corner::BR => child_elements.3.push(Entry{ loc, value }),
                }
                let newbounds = bounds.split();
                let children = (
                    Node::Leaf { bounds: newbounds[0], elements: child_elements.0 },
                    Node::Leaf { bounds: newbounds[1], elements: child_elements.1 },
                    Node::Leaf { bounds: newbounds[2], elements: child_elements.2 },
                    Node::Leaf { bounds: newbounds[3], elements: child_elements.3 },
                );
                (bounds, children)
            }
        } else {
            unreachable!();
        };

        let ptr0 = self.nodes.len() as NodePtr;
        let (c0, c1, c2, c3) = children;
        self.nodes.push(c0);
        self.nodes.push(c1);
        self.nodes.push(c2);
        self.nodes.push(c3);

        self.nodes[ptr as usize] = Node::Branch {
            bounds: bounds,
            children: [
                ptr0 + 0,
                ptr0 + 1,
                ptr0 + 2,
                ptr0 + 3,
            ]
        }
    }

    /// Get the value stored at a particular location.
    pub fn get(&mut self, loc: Loc) -> Option<&T> {
        let (idx, _) = self.lookup(loc);
        let node = &mut self.nodes[idx as usize];
        if let &mut Node::Leaf { ref mut elements, .. } = node {
            //println!("get {:?} bounds: {:?}, {:?}", loc, bounds, elements);
            elements.iter().filter(|e| e.loc == loc).next().map(|e| &e.value)
        } else {
            unreachable!()
        }
    }

    /// Remove a location and its attached data from the quadtree.
    /// If the location is found, its attached data is returned; if it is not 
    /// present, None is returned.
    pub fn remove(&mut self, loc: Loc) -> Option<T> {
        let (idx, _) = self.lookup(loc);
        let node = &mut self.nodes[idx as usize];
        if let &mut Node::Leaf { ref mut elements, .. } = node {
            let idx = elements.iter().position(|e| e.loc == loc);
            idx.and_then(|idx| remove_unordered(elements, idx).map(|e| e.value))
        } else {
            unreachable!()
        }
    }

    fn check(&self) {
        for node in self.nodes.iter() {
            if let &Node::Leaf { bounds, ref elements, .. } = node {
                bounds.check();
                for elem in elements {
                    assert!(bounds.contains(elem.loc));
                }
            } else if let &Node::Branch { bounds, .. } = node {
                bounds.check();
            }
        }
    }

    /// Query all values in a range, calling "callback" for each one.
    /// Range defines the region:
    /// [loc.0 - range, loc.0 + range] x [loc.1 - range, loc.1 + range]
    pub fn range_query<F: FnMut(Loc, &T)>(&self, loc: Loc, range: Coord, cb: F) {
        self.rect_query(
            loc.0 - range,
            loc.0 + range,
            loc.1 - range,
            loc.1 + range,
            cb
        )
    }

    /// Query all values in a rectangle, calling "callback" for each one.
    pub fn rect_query<F: FnMut(Loc, &T)>(&self, minx: Coord, maxx: Coord, miny: Coord, maxy: Coord, mut cb: F) {
        // this should probably be an iterator but haha have fun writing that code
        let bounds = Bounds {minx, maxx, miny, maxy};
        self._rect_query(bounds, 0, &mut cb);
    }

    fn _rect_query<F: FnMut(Loc, &T)>(&self, bounds: Bounds, ptr: NodePtr, cb: &mut F) {
        match &self.nodes[ptr as usize] {
            &Node::Leaf { ref elements, .. } => {
                for element in elements {
                    if bounds.contains(element.loc) {
                        cb(element.loc, &element.value);
                    }
                }
            },
            &Node::Branch { ref children, bounds: branch_bounds } => {
                let child_bounds = branch_bounds.split();
                if bounds.overlaps(idx!(child_bounds, 0)) {
                    self._rect_query(bounds, *idx!(children, 0), cb);
                }
                if bounds.overlaps(idx!(child_bounds, 1)) {
                    self._rect_query(bounds, *idx!(children, 1), cb);
                }
                if bounds.overlaps(idx!(child_bounds, 2)) {
                    self._rect_query(bounds, *idx!(children, 2), cb);
                }
                if bounds.overlaps(idx!(child_bounds, 3)) {
                    self._rect_query(bounds, *idx!(children, 3), cb);
                }
            }
        }
    }

    ///// "Garbage collect" unused nodes in the tree; that is, if a branch node has fewer than
    ///// SPLIT_THRESH children, it will be transformed into a leaf node.
    //pub fn gc(&mut self) {
    //    self._gc(0);
    //}

    //fn _gc(&mut self, ptr: NodePtr) {
    //    let node = &mut self.nodes[ptr as usize];
    //    match node {
    //        &mut Node::Leaf { .. } 
    //    }
    //}
}

// Remove an element from a vector, not preserving the vector's order.
// O(1)
fn remove_unordered<A: Array>(vec: &mut SmallVec<A>, element: usize) -> Option<A::Item> {
    if vec.len() > 1 {
        let idx = vec.len() - 1;
        vec.swap(element, idx)
    } 
    vec.pop()
}

#[cfg(test)]
mod test {
    use super::*;
    use fnv::FnvHashSet;

    #[test]
    fn bounds() {
        let bounds = Bounds {
            minx: 12, maxx: 37,
            miny: 55, maxy: 200
        };
        for x in 0..255 {
            for y in 0..255 {
                let inbounds = bounds.minx <= x && x <= bounds.maxx
                    && bounds.miny <= y && y <= bounds.maxy;
                assert_eq!(bounds.contains((x,y)), inbounds);
            }
        }
        let split = bounds.split();
        for x in bounds.minx..bounds.maxx+1 {
            for y in bounds.miny..bounds.maxy+1 {
                let corner = bounds.child((x,y));
                for i in 0..4u8 {
                    assert_eq!(split[i as usize].contains((x,y)), corner as u8 == i);
                }
            }
        }
    }

    fn bounds_overlap() {
        let bounds_a = Bounds {
            minx: 12, maxx: 37,
            miny: 55, maxy: 200
        };
        let bounds_b = Bounds {
            minx: 25, maxx: 27,
            miny: 190, maxy: 255
        };
        let bounds_c = Bounds {
            minx: 0, maxx: 128,
            miny: 210, maxy: 220
        };
        assert!(bounds_a.overlaps(&bounds_b));
        assert!(bounds_b.overlaps(&bounds_c));
        assert!(!bounds_a.overlaps(&bounds_c));
    }

    #[test]
    fn basics() {
        let mut q = Quadtree::new();
        q.insert((0,1), "banana");
        q.insert((233,57), "ocelot");
        q.check();
        assert_eq!(q.get((0,1)), Some(&"banana"));
        assert_eq!(q.get((233,57)), Some(&"ocelot"));
        assert_eq!(q.get((25,0)), None);
        assert_eq!(q.remove((0,1)), Some("banana"));
        assert_eq!(q.remove((0,1)), None);
    }

    #[test]
    fn tile() {
        let mut q = Quadtree::new();
        let hash = |x,y| x as usize + y as usize * 100;
        for x in 0..100 {
            for y in 0..100 {
                q.insert((x,y), hash(x,y));
            }
        }
        q.check();
        for x in 0..100 {
            for y in 0..100 {
                assert_eq!(q.get((x,y)), Some(&hash(x,y)));
            }
        }
        let mut count = 0;
        let mut visited = FnvHashSet::default();
        let bounds = Bounds {minx: 3, maxx: 97, miny: 66, maxy: 78};
        q.rect_query(bounds.minx, bounds.maxx, bounds.miny, bounds.maxy, |loc, &value| {
            assert!(bounds.contains(loc));
            assert_eq!(value, hash(loc.0, loc.1));
            assert!(!visited.contains(&loc));
            visited.insert(loc);
            count += 1;
        });
        assert_eq!(count, (bounds.maxx - bounds.minx + 1) as usize
                        * (bounds.maxy - bounds.miny + 1) as usize);
    }
}