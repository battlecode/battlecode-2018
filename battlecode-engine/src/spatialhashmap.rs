#![allow(dead_code)]

//! A reasonably fast quadtree implementation.
use smallvec::{SmallVec, Array};
use super::location::{MapLocation, Planet};

/// A coordinate representable in the quadtree.
pub type Coord = u8;
/// A location in the quadtree.
pub type Loc = (Coord, Coord);
/// The maximum possible coordinate.
pub const MIN_COORD: Coord = 0;
/// The maximum possible coordinate.
pub const MAX_COORD: Coord = 63;
/// The size of the world represented by the quadtree.
pub const SIZE: usize = 64;

const BUCKET_SIZE: usize = 8;
const BUCKETS_PER_DIM: usize = SIZE/BUCKET_SIZE;
const BUCKET_COUNT: usize = BUCKETS_PER_DIM*BUCKETS_PER_DIM;

/// A "pointer" to a node; actually an index into a backing vector.
type NodePtr = u16;

macro_rules! idx {
    ($arr:expr, $i:expr) => {
        //&$arr[$i]
        unsafe { $arr.get_unchecked($i) }
    };
}
macro_rules! idx_mut {
    ($arr:expr, $i:expr) => {
        //&mut $arr[$i]
        unsafe { $arr.get_unchecked_mut($i) }
    };
}

/// An entry in the quadtree.
#[derive(Debug, Clone)]
struct Entry<T: Clone> {
    loc: Loc,
    value: T
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
    fn contains(&self, loc: Loc) -> bool {
        self.minx <= loc.0 &&
        self.miny <= loc.1 &&
        self.maxx >= loc.0 &&
        self.maxy >= loc.1
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
}

/// A quadtree.
/// The quadtree maps locations - (u8, u8) tuples - to arbitrary values.
/// If your data doesn't fit in the range (0, 255)... make it fit.
/// Multiple keys cannot be stored in the same location.
#[derive(Clone)]
pub struct SpatialHashMap<T: Clone> {
    /// indexed as [y][x]
    buckets: Vec<SmallVec<[Entry<T>; 8]>>
}
impl <T: Clone> SpatialHashMap<T> {
    /// Make a new quadtree.
    pub fn new() -> SpatialHashMap<T> {
        let mut buckets = Vec::with_capacity(BUCKET_COUNT);
        for _ in 0..BUCKET_COUNT {
            buckets.push(SmallVec::new());
        }

        SpatialHashMap { buckets }
    }

    /// Lookup the bucket for a given location.
    fn bucket(&self, loc: Loc) -> usize {
        (loc.0 as usize / BUCKET_SIZE) * BUCKETS_PER_DIM +
            (loc.1 as usize / BUCKET_SIZE)
    }
    
    /// Lookup the bucket bounds for a given location.
    fn bounds(&self, loc: Loc) -> Bounds {
        let minx = (loc.0 as usize / BUCKET_SIZE) * BUCKET_SIZE;
        let miny = (loc.1 as usize / BUCKET_SIZE) * BUCKET_SIZE;
        let maxx = minx + BUCKET_SIZE - 1;
        let maxy = miny + BUCKET_SIZE - 1;

        Bounds { minx: minx as Coord, miny: miny as Coord, maxx: maxx as Coord, maxy: maxy as Coord }
    }

    #[inline(always)]
    fn oob(&self, loc: Loc) -> bool {
        return loc.0 >= MAX_COORD || loc.1 >= MAX_COORD;
    }

    /// Insert a value into the quadtree.
    pub fn insert(&mut self, loc: Loc, value: T) {
        if self.oob(loc) {
            return;
        }

        let idx = self.bucket(loc);
        if cfg!(debug) {
            for elem in &self.buckets[idx] {
                assert!(elem.loc != loc);
            }
            let bounds = self.bounds(loc);
            assert!(bounds.contains(loc), "{:?} {:?}", bounds, loc);
        }
        idx_mut!(self.buckets, idx).push(Entry { loc, value });
    }

    /// Get the value stored at a particular location.
    pub fn get(&self, loc: Loc) -> Option<&T> {
        if self.oob(loc) {
            return None;
        }
        
        let idx = self.bucket(loc);
        idx!(self.buckets, idx).iter().filter(|e| e.loc == loc).next().map(|e| &e.value)
    }

    /// Remove a location and its attached data from the quadtree.
    /// If the location is found, its attached data is returned; if it is not 
    /// present, None is returned.
    pub fn remove(&mut self, loc: Loc) -> Option<T> {
        if self.oob(loc) {
            return None;
        }
        let idx = self.bucket(loc);
        let bucket = idx_mut!(self.buckets, idx);
        bucket.iter()
            .position(|e| e.loc == loc)
            .and_then(|idx| remove_unordered(bucket, idx).map(|e| e.value))
    }

    fn check(&self) {
        for x in 0..BUCKETS_PER_DIM {
            for y in 0..BUCKETS_PER_DIM {
                let loc = ((x * BUCKET_SIZE) as Coord, (y * BUCKET_SIZE) as Coord);
                let bounds = self.bounds(loc);
                for entry in &self.buckets[self.bucket(loc)] {
                    assert!(bounds.contains(entry.loc), "{:?} {:?}", bounds, entry.loc);
                }
            }
        }
    }

    /// Query all values in a rough circle, calling "callback" for each one.
    /// Range defines the region:
    /// [loc.0 - range, loc.0 + range] x [loc.1 - range, loc.1 + range]
    pub fn range_query<F: FnMut(Loc, &T)>(&self, loc: Loc, distsq: Coord, mut cb: F) {
        let bound = |x| i32::min(i32::max(x, 0), MAX_COORD as i32) as Coord;
        let distsq = distsq as i32;
        let range = (f32::sqrt(distsq as f32) + 1.) as i32;

        let minx = bound(loc.0 as i32 - range);
        let maxx = bound(loc.0 as i32 + range);
        let miny = bound(loc.1 as i32 - range);
        let maxy = bound(loc.1 as i32 + range);

        let mut x = (minx as usize / BUCKET_SIZE) * BUCKET_SIZE;
        while x < maxx as usize {
            let mut y = (miny as usize / BUCKET_SIZE) * BUCKET_SIZE;
            while y < maxy as usize {
                let idx = self.bucket((x as Coord, y as Coord));
                for entry in idx!(self.buckets, idx) {
                    let dx = entry.loc.0 as i32 - loc.0 as i32;
                    let dy = entry.loc.1 as i32 - loc.1 as i32;
                    if dx * dx + dy * dy <= distsq {
                        cb(entry.loc, &entry.value);
                    }
                }
                y += BUCKET_SIZE;
            }
            x += BUCKET_SIZE;
        }

    }

    /// Query all values in a rectangle, calling "callback" for each one.
    pub fn rect_query<F: FnMut(Loc, &T)>(&self, minx: Coord, maxx: Coord, miny: Coord, maxy: Coord, mut cb: F) {
        // this should probably be an iterator but haha have fun writing that code

        let bounds = Bounds {minx, maxx, miny, maxy};

        let mut x = (minx as usize / BUCKET_SIZE) * BUCKET_SIZE;
        while x < maxx as usize {
            let mut y = (miny as usize / BUCKET_SIZE) * BUCKET_SIZE;
            while y < maxy as usize {
                let idx = self.bucket((x as Coord, y as Coord));
                for entry in idx!(self.buckets, idx) {
                    if bounds.contains(entry.loc) {
                        cb(entry.loc, &entry.value);
                    }
                }
                y += BUCKET_SIZE;
            }
            x += BUCKET_SIZE;
        }
    }
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

#[derive(Clone)]
pub struct MapLocationMap<T: Clone> {
    earth: SpatialHashMap<T>,
    mars: SpatialHashMap<T>,
    earth_origin: MapLocation,
    mars_origin: MapLocation
}

impl<T: Clone> MapLocationMap<T> {
    pub fn new(earth_origin: MapLocation, mars_origin: MapLocation) -> MapLocationMap<T> {
        debug_assert!(earth_origin.planet == Planet::Earth);
        debug_assert!(mars_origin.planet == Planet::Mars);
        MapLocationMap {
            earth: SpatialHashMap::new(),
            mars: SpatialHashMap::new(),
            earth_origin,
            mars_origin
        }
    }

    #[inline(always)]
    fn convert(&self, loc: MapLocation) -> (Planet, Loc) {
        if loc.planet == Planet::Earth {
            (loc.planet, ((loc.x - self.earth_origin.x) as Coord, (loc.y - self.earth_origin.y) as Coord))
        } else {
            (loc.planet, ((loc.x - self.mars_origin.x) as Coord, (loc.y - self.mars_origin.y) as Coord))
        }
    }
    
    pub fn insert(&mut self, loc: MapLocation, value: T) {
        let (planet, loc) = self.convert(loc);
        if planet == Planet::Earth {
            self.earth.insert(loc, value);
        } else {
            self.mars.insert(loc, value);
        }
    }

    pub fn get(&self, loc: MapLocation) -> Option<&T> {
        let (planet, loc) = self.convert(loc);
        if planet == Planet::Earth {
            self.earth.get(loc)
        } else {
            self.mars.get(loc)
        }
    }

    #[inline(always)]
    pub fn contains(&self, loc: MapLocation) -> bool {
        self.get(loc).is_some()
    }

    pub fn remove(&mut self, loc: MapLocation) -> Option<T> {
        let (planet, loc) = self.convert(loc);
        if planet == Planet::Earth {
            self.earth.remove(loc)
        } else {
            self.mars.remove(loc)
        }
    }

    pub fn range_query<F: FnMut(MapLocation, &T)>(&self, loc: MapLocation, range: Coord, mut cb: F) {
        let origin = if loc.planet == Planet::Earth {
            self.earth_origin
        } else {
            self.mars_origin
        };
        let cb = |loc: Loc, val: &T| {
            let loc = MapLocation {
                x: origin.x + loc.0 as i32,
                y: origin.y + loc.1 as i32,
                planet: origin.planet
            };
            cb(loc, val);
        };
        let (planet, loc) = self.convert(loc);
        if planet == Planet::Earth {
            self.earth.range_query(loc, range, cb);
        } else {
            self.mars.range_query(loc, range, cb);
        }
    }
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
        let mut q = SpatialHashMap::new();
        q.insert((0,1), "banana");
        q.insert((33,57), "ocelot");
        q.check();
        assert_eq!(q.get((0,1)), Some(&"banana"));
        assert_eq!(q.get((33,57)), Some(&"ocelot"));
        assert_eq!(q.get((25,0)), None);
        assert_eq!(q.remove((0,1)), Some("banana"));
        assert_eq!(q.remove((0,1)), None);
    }

    #[test]
    fn tile() {
        let mut q = SpatialHashMap::new();
        let hash = |x,y| x as usize + y as usize * 64;
        for x in 0..63 {
            for y in 0..63 {
                q.insert((x,y), hash(x,y));
            }
        }
        q.check();
        for x in 0..63 {
            for y in 0..63 {
                assert_eq!(q.get((x,y)), Some(&hash(x,y)));
            }
        }
        let mut count = 0;
        let mut visited = FnvHashSet::default();
        let bounds = Bounds {minx: 3, maxx: 57, miny: 16, maxy: 28};
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